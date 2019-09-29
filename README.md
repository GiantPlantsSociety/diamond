# Diamond

[![build status](https://github.com/GiantPlantsSociety/diamond/workflows/Rust/badge.svg)](https://github.com/GiantPlantsSociety/diamond/actions)
[![codecov](https://codecov.io/gh/GiantPlantsSociety/diamond/branch/master/graph/badge.svg)](https://codecov.io/gh/GiantPlantsSociety/diamond)
[![Docker](https://img.shields.io/docker/cloud/build/giantplantssociety/diamond.svg)](https://hub.docker.com/r/giantplantssociety/diamond/)

#### Distribution

##### Docker

Diamond build is deployed to [docker-hub](https://hub.docker.com/r/giantplantssociety/diamond/). It contains diamond-server, diamond-api, diamond-pipe and whisper utilities.

To run diamond-server on any system that has docker:

```
docker run -it giantplantssociety/diamond
```

To run diamond-api:

```
docker run -it giantplantssociety/diamond diamond-api
```

##### Setup

Install [rrdtool](https://oss.oetiker.ch/rrdtool/index.en.html):
- Linux: `<package manager> install librrd-dev rrdtool`
- MacOS: `brew install rrdtool`
- FreeBSD: `pkg install rrdtool`

##### Build

```
cargo build
```

##### Test

```
cargo test
```

##### Run

##### Diamond-api

```
./diamond-api -p 8080 -d data -f
```

```
USAGE:
    diamond-api [FLAGS] [OPTIONS]

FLAGS:
    -f, --force      Force to create data directory if it is absent
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --data-dir <path>    Path to data directory, default value is a current directory [default: .]
    -p, --port <port>        Port to listen on [default: 8080]
```

##### Diamond-server

`./diamond-server -c src/config.toml`

Default config from `diamond/src/config.toml` is already embedded

```
USAGE:
    diamond-server [FLAGS] [OPTIONS]

FLAGS:
    -g               Generate default config file
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Path to config file
```

##### Diamond-pipe

`echo "this.is.correct 123 1545775266" | ./diamond-pipe data-dir 60:100`

```
Receive metrics from pipe

USAGE:
    diamond-pipe [OPTIONS] <path> <retentions>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --aggregationMethod <aggregation_method>
            Default function to use when aggregating values (average, sum, last, max, min, avg_zero, absmax, absmin)
            [default: average]
        --xFilesFactor <x_files_factor>             Default value for the xFilesFactor for new files [default: 0.5]

ARGS:
    <path>             Path to the directory with data files
    <retentions>...     Default retentions for new files
                       Specify lengths of time, for example:
                       60:1440      60 seconds per datapoint, 1440 datapoints = 1 day of retention
                       15m:8        15 minutes per datapoint, 8 datapoints = 2 hours of retention
                       1h:7d        1 hour per datapoint, 7 days of retention
                       12h:2y       12 hours per datapoint, 2 years of retention
```
