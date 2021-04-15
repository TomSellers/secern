use regex::RegexSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::time::Instant;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

use env_logger::Env;
use log::{error, info};

extern crate serde;

use serde::{Deserialize, Serialize};

// Operational object
#[derive(Debug)]
struct FilterConfig {
    name: String,
    file_name: String,
    file: Option<BufWriter<std::fs::File>>,
    regex_set: RegexSet,
    invert: bool,
}

// Config structures from the YAML config file
#[derive(Deserialize, Debug, Serialize)]
struct SinkConfig {
    name: String,
    file_name: String,
    patterns: Vec<String>,
    invert: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct SinkList {
    sinks: Vec<SinkConfig>,
}

fn load_config(config: &str) -> Vec<FilterConfig> {
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
fn generate_config(file_name: &str) {
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

    let all_sinks = vec![first, second];

    let yaml_string = serde_yaml::to_string(&all_sinks).unwrap();

    // FIXFIX add error handling
    file.write_all(yaml_string.as_bytes()).unwrap();
    file.flush().unwrap();
    std::process::exit(0);
}

// final_flush ensures that all buffered file output is written before bailing
fn final_flush(
    mut filters: Vec<FilterConfig>,
    mut stdio_writer: std::io::BufWriter<std::io::Stdout>,
) {
    for filter in &mut filters {
        match &mut filter.file {
            None => (),
            Some(out_file) => {
                match out_file.flush() {
                    Ok(_data) => (),
                    Err(e) => {
                        error!(
                            "Error flushing final data to output file '{}' for sink named '{}' due to error: {}",
                            filter.file_name, filter.name, e
                        );
                        std::process::exit(1);
                    }
                };
            }
        }
    }

    stdio_writer.flush().unwrap();
}

//FIXFIX - Handle Ctrl-C
fn main() {
    let matches =
        App::new(crate_name!())
            .version(crate_version!())
            .author(crate_authors!(","))
            .about(crate_description!())
            .arg(
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("Specifies the YAML config file")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("generate")
                    .short("g")
                    .long("gen-template")
                    .value_name("FILE")
                    .help("Generates an example YAML config file and exits")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("no-stdout")
                    .short("n")
                    .long("no-stdout")
                    .help("Disables emmitting unfiltered data on STDOUT"),
            )
            .arg(Arg::with_name("quiet").short("q").long("quiet").help(
                "Disables emmitting info level log events (version, run time, etc) on STDERR",
            ))
            .get_matches();

    if let Some(t) = matches.value_of("generate") {
        generate_config(t);
    }

    let config: &str;
    match matches.value_of("config") {
        Some(s) => config = s,
        None => {
            // clap ensures that the value of `config` is populated but handle
            // missing values here anyway.
            error!("Please specify the configuration file!");
            std::process::exit(1)
        }
    }

    let log_level: String;
    if matches.is_present("quiet") {
        log_level = String::from("warn");
    } else {
        log_level = String::from("info");
    }
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    info!("{} {}", crate_name!(), crate_version!());

    info!("Loading configuration file: {}", config);
    let mut filters = load_config(config);

    info!("Starting data processing.");
    let start = Instant::now();
    let mut stdio_writer = BufWriter::with_capacity(4096 * 1024, io::stdout());

    let stdin = BufReader::with_capacity(64 * 1024, io::stdin());
    let mut found_match: bool;
    for entry in stdin.lines() {
        let line = entry.unwrap();
        found_match = false;

        for filter in &mut filters {
            let mut matched: bool = filter.regex_set.is_match(&line);
            if filter.invert {
                matched = !matched;
            }
            if matched {
                match &mut filter.file {
                    None => (),
                    Some(out_file) => {
                        match out_file.write_all(line.as_bytes()) {
                            Ok(_) => (),
                            Err(e) => {
                                error!(
                                    "Unable to write to output file '{}' for sink named '{}' due to error: {}",
                                    filter.file_name, filter.name, e
                                );
                                std::process::exit(1);
                            }
                        };

                        match out_file.write_all(b"\n") {
                            Ok(_) => (),
                            Err(e) => {
                                error!(
                                    "Unable to write to output file '{}' for sink named '{}' due to error: {}",
                                    filter.file_name, filter.name, e
                                );
                                std::process::exit(1);
                            }
                        };
                    }
                }
                found_match = true;
                break;
            }
        }

        if !found_match && !matches.is_present("no-stdout") {
            // TODO: Error handling when writing to STDOUT + broken pipe (head -n 10)
            //       thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)', library/std/src/io/stdio.rs:940:9
            //       Consider how to close down the various filter files correctly
            //       https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#matching-on-different-errors

            // It is faster to use two writes (data followed by \n) than
            // using writeln!()
            match stdio_writer.write_all(line.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    error!("Unable to write data to STDOUT due to error: {}", e);
                    std::process::exit(1);
                }
            };

            match stdio_writer.write_all(b"\n") {
                Ok(_) => (),
                Err(e) => {
                    error!("Unable to write data to STDOUT due to error: {}", e);
                    std::process::exit(1);
                }
            };
        }
    }

    final_flush(filters, stdio_writer);

    let duration = start.elapsed();
    info!("Ending data processing. Time elapsed was: {:?}", duration);
}
