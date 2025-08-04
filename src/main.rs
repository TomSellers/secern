mod config;

use std::fs;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};

use std::time::Instant;

use clap::{Arg, ArgAction, Command, crate_authors, crate_description, crate_name, crate_version};

use env_logger::Env;
use log::{error, info};

// final_flush ensures that all buffered file output is written before bailing
fn final_flush(
    mut filters: Vec<config::FilterConfig>,
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

fn main() {
    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!(","))
        .about(crate_description!())
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Specifies the YAML config file")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("generate")
                .short('g')
                .long("gen-template")
                .value_name("FILE")
                .help("Generates an example YAML config file and exits")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("validate-only")
                .short('v')
                .long("validate-only")
                .help("Validate that the config file specified by -c is correctly formed.")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-stdout")
                .short('n')
                .long("no-stdout")
                .help("Disables emmitting unfiltered data on STDOUT")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Disables emmitting info level log events (version, run time, etc) on STDERR")
                .action(ArgAction::SetTrue),
        )
        .get_matches();

    // Initialize logging
    let log_level = if matches.get_flag("quiet") { "warn" } else { "info" };
    env_logger::Builder::from_env(Env::default().default_filter_or(log_level)).init();

    info!("{} {}", crate_name!(), crate_version!());

    if let Some(t) = matches.get_one::<String>("generate") {
        config::generate_config(t);
    }

    let config: &str = match matches.get_one::<String>("config") {
        Some(s) => s,
        None => {
            // clap ensures that the value of `config` is populated but handle
            // missing values here anyway.
            error!("Please specify the configuration file!");
            std::process::exit(1)
        }
    };

    let validate_only: bool = matches.get_flag("validate-only");

    info!("Loading configuration file: {config}");

    let config_data = fs::read_to_string(config);
    let config_data = match config_data {
        Ok(data) => data,
        Err(e) => {
            error!("Unable to open specified configuration file ({config}) due to error: {e}");
            std::process::exit(1);
        }
    };

    let mut filters = config::process_config(config, config_data, validate_only);

    if validate_only {
        info!("Configuration summary");
        config::display_config_summary(filters);
        std::process::exit(0)
    }

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
                            Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
                                std::process::exit(0);
                            }
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

        if !found_match && !matches.get_flag("no-stdout") {
            // TODO: Error handling when writing to STDOUT + broken pipe (head -n 10)
            //       thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)', library/std/src/io/stdio.rs:940:9
            //       Consider how to close down the various filter files correctly
            //       https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#matching-on-different-errors

            // It is faster to use two writes (data followed by \n) than
            // using writeln!()
            match stdio_writer.write_all(line.as_bytes()) {
                Ok(_) => (),
                Err(e) => {
                    error!("Unable to write data to STDOUT due to error: {e}");
                    std::process::exit(1);
                }
            };

            match stdio_writer.write_all(b"\n") {
                Ok(_) => (),
                Err(e) => {
                    error!("Unable to write data to STDOUT due to error: {e}");
                    std::process::exit(1);
                }
            };
        }
    }

    final_flush(filters, stdio_writer);

    let duration = start.elapsed();
    info!("Ending data processing. Time elapsed was: {duration:?}");
}
