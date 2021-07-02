use log::{error, info};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

// Operational object
#[derive(Debug)]
pub struct FilterConfig {
    pub name: String,
    pub file_name: String,
    pub file: Option<BufWriter<std::fs::File>>,
    pub regex_set: RegexSet,
    pub invert: bool,
}

// Config structures from the YAML config file
#[derive(Deserialize, Debug, Serialize)]
pub struct SinkConfig {
    name: String,
    file_name: String,
    patterns: Vec<String>,
    invert: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct SinkList {
    sinks: Vec<SinkConfig>,
}

pub fn load_config(config: &str, validate_only: bool) -> Vec<FilterConfig> {
    let mut filters: Vec<FilterConfig> = Vec::new();

    let f = File::open(config);
    let f = match f {
        Ok(file) => file,
        Err(e) => {
            error!(
                "Unable to open specified configuration file ({}) due to error: {}",
                config, e
            );
            std::process::exit(1);
        }
    };

    let sink_list = serde_yaml::from_reader(f);
    let sink_list: SinkList = match sink_list {
        Ok(data) => data,
        Err(e) => {
            error!(
                "Error parsing configuration file ({}) due to error: {}",
                config, e
            );
            std::process::exit(1);
        }
    };

    // Make a pass through and verify that all the regex compiles.  By doing it
    // this way we can print all of the errors at once so users can fix them
    // at one time instead of having to fix one, and rerun to check the rest.
    let mut config_error: bool = false;
    for sink in &sink_list.sinks {
        match RegexSet::new(&sink.patterns) {
            Ok(_) => (),
            Err(e) => {
                error!(
                    "Error parsing Regex pattern in sink named '{}' due to error: {}",
                    sink.name, e
                );
                config_error = true;
            }
        }
    }

    if config_error {
        std::process::exit(1);
    }

    for sink in sink_list.sinks {
        let filter_set = RegexSet::new(&sink.patterns);
        let filter_set: RegexSet = match filter_set {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Error parsing Regex pattern in sink named '{}' due to error: {}",
                    sink.name, e
                );
                std::process::exit(1);
            }
        };

        let file: Option<BufWriter<std::fs::File>>;
        if sink.file_name == "null" || validate_only {
            file = None;
        } else {
            let path = Path::new(&sink.file_name);

            let prefix = path.parent().unwrap();
            if !prefix.exists() {
                match std::fs::create_dir_all(prefix) {
                    Ok(_) => (),
                    Err(e) => {
                        error!(
                            "Output file creation failed while creating directory '{}' due to error: {}",
                            prefix.display(), e);
                        std::process::exit(1);
                    }
                }
            }

            file = match File::create(&path) {
                Ok(file) => Some(std::io::BufWriter::new(file)),
                Err(e) => {
                    error!(
                        "Unable to create output file '{}' for sink named '{}' due to error: {}",
                        path.display(),
                        sink.name,
                        e
                    );
                    std::process::exit(1);
                }
            };
        }

        let invert: bool;
        match sink.invert {
            None => invert = false,
            Some(val) => invert = val,
        }

        let temp = FilterConfig {
            name: sink.name,
            file_name: sink.file_name,
            file,
            regex_set: filter_set,
            invert,
        };

        filters.push(temp);
    }

    filters
}

// generate_config emits a sample YAML configuration file
pub fn generate_config(file_name: &str) {
    let path = Path::new(&file_name);

    let prefix = path.parent().unwrap();
    if !prefix.exists() {
        match std::fs::create_dir_all(prefix) {
            Ok(_) => (),
            Err(e) => {
                error!(
                    "Template generation failed while creating directory '{}' due to error: {}",
                    prefix.display(),
                    e
                );
                std::process::exit(1);
            }
        }
    }

    //FIXFIX - ERROR handling - if path doesn't exist
    let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => file,
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => {
                error!(
                    "The specified template file '{}' already exists and will NOT be overwritten.",
                    file_name,
                );
                std::process::exit(1);
            }
            _ => {
                error!(
                    "Unable to create template file '{}' due to error: {}",
                    file_name, e,
                );
                std::process::exit(1);
            }
        },
    };

    let first = SinkConfig {
        name: "first_sink".to_string(),
        file_name: "first_output.txt".to_string(),
        patterns: vec!["^[a-zA-Z0-9]+$".to_string()],
        invert: None,
    };

    let second = SinkConfig {
        name: "second_sink".to_string(),
        file_name: "second_output.txt".to_string(),
        patterns: vec!["😎*".to_string()],
        invert: None,
    };

    let mut config = HashMap::new();
    config.insert(String::from("sinks"), vec![first, second]);

    let yaml_string = serde_yaml::to_string(&config).unwrap();

    // FIXFIX add error handling
    file.write_all(yaml_string.as_bytes()).unwrap();
    file.flush().unwrap();
    std::process::exit(0);
}

pub fn display_config_summary(filters: Vec<FilterConfig>) {
    let mut name_len = 0;
    let mut file_name_len = 0;

    for filter in &filters {
        if filter.name.chars().count() > name_len {
            name_len = filter.name.chars().count()
        }

        if filter.file_name.chars().count() > name_len {
            file_name_len = filter.file_name.chars().count()
        }
    }

    for filter in filters {
        info!(
            "Sink name: {:<name_len$} Output file name: {:<file_name_len$} Invert match: {}",
            filter.name,
            filter.file_name,
            filter.invert,
            name_len = name_len + 2,
            file_name_len = file_name_len + 2,
        )
    }
}
