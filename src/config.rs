use log::error;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
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

pub fn load_config(config: &str) -> Vec<FilterConfig> {
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
        if sink.file_name == "null" {
            file = None;
        } else {
            let path = Path::new(&sink.file_name);
            let display = path.display();

            //FIXFIX - ERROR handling - if path doesn't exist
            file = match File::create(&path) {
                Ok(file) => Some(std::io::BufWriter::new(file)),
                Err(e) => {
                    error!(
                        "Unable to create output file '{}' for sink named '{}' due to error: {}",
                        display, sink.name, e
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

    //FIXFIX - ERROR handling - if path doesn't exist
    let mut file = match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            error!(
                "Unable to create template file '{}' due to error: {}",
                file_name, e
            );
            std::process::exit(1);
        }
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
