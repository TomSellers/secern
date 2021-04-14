# secern

`secern` is a command line string filtering program. It accepts data on STDIN, uses regular expressions to filter the data, and returns unfiltered data on STDOUT. It can have multiple sinks (outputs) and can use multiple regex patterns per sink.

The value that `secern` provides is

- multiple regular expression patterns can be specified the configuration file. Data will be filtered if it matches any pattern. Data will be filtered based on the first pattern that it matches.

- the filtered data can be saved to a file defined in the configuration file.

## Usage

Bash

```bash
# Compiled
head -n 100 testing/source_data/sample.txt | ./secern -config filter.yaml

# From source
head -n 100 testing/source_data/sample.txt | cargo run --release -- --config filter.yaml
 ```

PowerShell

```powershell
# Compiled
Get-Content -Head 100 -encoding UTF8 .\testing\source_data\sample.txt | secern -config filter.yaml

# From source
Get-Content -Head 100 -encoding UTF8 .\testing\source_data\sample.txt | cargo run --release -- --config filter.yaml
```

## Building

```shell
cargo build --release
```

## TODO

- FIXFIX: Handle SIGTERM / Ctrl-C
- FIXFIX: Re-implement tests after porting to Rust
- FIXFIX: Handle if output directory is missing
- FIXFIX: Warning about paths in the config and needing to use / or autofix
- FEATURE: Autodetect when to use more than one CPU based on regex parse time
- CLI: flag to validate config files
