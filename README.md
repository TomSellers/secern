# secern

## Overview

`secern` is a command line string sifting program. It accepts data on STDIN,
uses regular expressions to identify data of interest which it saves in the
specified output files, and returns unfiltered data on STDOUT. It can have
multiple sinks (outputs) and can use multiple regex patterns per sink.

As an example, if you had a list of DNS hostnames and you could use the
following config to:

- save all of the names ending in `.net` in `dot_net.txt`
- save all of the names ending in `.com` in `dot_com.txt`
- save all of the names ending in `.gov` or `.gov.uk` to `government.txt`
- emit all lines that don't match either of the above on STDOUT

Example config

```yaml
---
sinks:
  - name: dot_net_domains
    file_name: dot_net.txt
    patterns:
      - '\.net$'
  - name: dot_com_domains
    file_name: dot_com.txt
    patterns:
      - '\.com$'
- name: government_domains
    file_name: government.txt
    patterns:
      - '\.gov$'
      - '\gov\.uk$'
```

If you don't want to emit anything on `STDOUT` you can use the `-n` flag to
disable it.

**NOTE**: I built `secern` to solve a problem and to learn Rust. There are likely
improvements that can be made. Please open an issue if you find a bug or have
recommendations for improvements.

## Performance

`secern` is pretty fast thanks to
[`regex::RegexSet`](https://docs.rs/regex/1.4.5/regex/struct.RegexSet.html) in
the Rust `regex` crate which allows multiple regular expression patterns to be
matched in a single pass.

In tests under WSL2 on Windows 10 I've processed 100 million lines with 2 sinks
and 9 regexs in about 25 seconds. Native Linux or Windows performance will be
higher.

General advice to improve performance:

- Limit the number of sinks. Multiple regex patterns on a single sink is fine.
- Prioritize the sinks and patterns within a sink that are most likely to match.
- Use `-n` to silence un-matched data on STDOUT if you don't need it.

## Usage

```shell
secern is a command line string sifting program that leverages
regex patterns defined in a configuration file to sift data into
specified output files.

USAGE:
    secern [FLAGS] --config <FILE>

FLAGS:
    -h, --help         Prints help information
    -n, --no-stdout    Disables emmitting unfiltered data on STDOUT
    -q, --quiet        Disables emmitting info level log events (version, run time, etc) on STDERR
    -V, --version      Prints version information

OPTIONS:
    -c, --config <FILE>    Specifies the YAML config file
```

Bash

```bash
# Compiled
head -n 100 sample.txt | ./secern -config filter.yaml

# From source
head -n 100 sample.txt | cargo run --release -- --config filter.yaml
 ```

PowerShell

```powershell
# Compiled
Get-Content -Head 100 -encoding UTF8 sample.txt | secern -config filter.yaml

# From source
Get-Content -Head 100 -encoding UTF8 sample.txt | cargo run --release -- --config filter.yaml
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
