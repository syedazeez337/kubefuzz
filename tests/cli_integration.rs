//! Integration tests for the `kf` binary.
//! These test CLI arg parsing and help output without needing a cluster.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_flag() {
    Command::cargo_bin("kf")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Fuzzy-first interactive Kubernetes resource navigator",
        ))
        .stdout(predicate::str::contains("--all-contexts"))
        .stdout(predicate::str::contains("--context"));
}

#[test]
fn version_flag() {
    Command::cargo_bin("kf")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("kf"));
}
