//! Integration tests for the `kf` binary.
//! These tests exercise CLI flags that exit immediately without a TTY or live cluster.
#![allow(deprecated)] // cargo_bin is deprecated only for custom build-dirs; standard builds are fine.

use assert_cmd::Command;
use predicates::prelude::*;

// ── --help ────────────────────────────────────────────────────────────────────

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
        .stdout(predicate::str::contains("--context"))
        .stdout(predicate::str::contains("--namespace"))
        .stdout(predicate::str::contains("--read-only"));
}

// ── --version ─────────────────────────────────────────────────────────────────

#[test]
fn version_flag() {
    Command::cargo_bin("kf")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("kf"));
}

// ── --completions ─────────────────────────────────────────────────────────────

#[test]
fn completions_bash() {
    Command::cargo_bin("kf")
        .unwrap()
        .args(["--completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_zsh() {
    Command::cargo_bin("kf")
        .unwrap()
        .args(["--completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn completions_fish() {
    Command::cargo_bin("kf")
        .unwrap()
        .args(["--completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ── --mangen ──────────────────────────────────────────────────────────────────

#[test]
fn mangen_flag() {
    Command::cargo_bin("kf")
        .unwrap()
        .arg("--mangen")
        .assert()
        .success()
        .stdout(predicate::str::contains("kf"));
}
