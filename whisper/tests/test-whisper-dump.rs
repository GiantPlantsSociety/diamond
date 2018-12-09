use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::Builder;
use unindent::unindent;

const NAME: &str = "whisper-dump";

#[test]
fn calling_without_args() {
    Command::cargo_bin(NAME)
        .unwrap()
        .assert()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("USAGE").from_utf8());
}

#[test]
fn calling_help() {
    Command::cargo_bin(NAME)
        .unwrap()
        .args(&["--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE").from_utf8())
        .stderr("");
}

#[test]
fn calling_with_invalid_path() {
    Command::cargo_bin(NAME)
        .unwrap()
        .args(&["invalid"])
        .assert()
        .code(1);
}

#[test]
fn calling_as_plain() {
    let filename = "dump.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()
        .unwrap()
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path).unwrap();

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                unindent(
                    "
Meta data:
  aggregation method: average
  max retention: 600
  xFilesFactor: 0.5",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                unindent(
                    "
Archive 0 info:
  offset: 40
  seconds per point: 60
  points: 5
  retention: 300
  size: 60",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                unindent(
                    "
Archive 1 info:
  offset: 100
  seconds per point: 120
  points: 5
  retention: 600
  size: 60",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                "
Archive 0 data:
0: 0,          0
1: 0,          0
2: 0,          0
3: 0,          0
4: 0,          0",
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                "
Archive 1 data:
0: 0,          0
1: 0,          0
2: 0,          0
3: 0,          0
4: 0,          0",
            ).from_utf8(),
        ).stderr("");
}

#[test]
fn calling_as_pretty() {
    let filename = "dump.wsp";

    let path = Builder::new()
        .prefix("whisper")
        .suffix(filename)
        .tempdir()
        .unwrap()
        .path()
        .to_path_buf();

    let file_path = PathBuf::new().join("data").join(filename);

    fs::copy(&file_path, &path).unwrap();

    Command::cargo_bin(NAME)
        .unwrap()
        .args(&[path.to_str().unwrap(), "--pretty", "--time-format", "%c"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains(
                unindent(
                    "
Meta data:
  aggregation method: average
  max retention: 600
  xFilesFactor: 0.5",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                unindent(
                    "
Archive 0 info:
  offset: 40
  seconds per point: 60
  points: 5
  retention: 300
  size: 60",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                unindent(
                    "
Archive 1 info:
  offset: 100
  seconds per point: 120
  points: 5
  retention: 600
  size: 60",
                ).as_str(),
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                "
Archive 0 data:
0: Thu Jan  1 00:00:00 1970,          0
1: Thu Jan  1 00:00:00 1970,          0
2: Thu Jan  1 00:00:00 1970,          0
3: Thu Jan  1 00:00:00 1970,          0
4: Thu Jan  1 00:00:00 1970,          0",
            ).from_utf8(),
        ).stdout(
            predicate::str::contains(
                "
Archive 1 data:
0: Thu Jan  1 00:00:00 1970,          0
1: Thu Jan  1 00:00:00 1970,          0
2: Thu Jan  1 00:00:00 1970,          0
3: Thu Jan  1 00:00:00 1970,          0
4: Thu Jan  1 00:00:00 1970,          0",
            ).from_utf8(),
        ).stderr("");
}
