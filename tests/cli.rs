use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn passing_include_ignored_flag_runs_ignored_tests() {
    Command::cargo_bin("cargo-testdox")
        .unwrap()
        .current_dir("testdata/testproj")
        .arg("testdox")
        .arg("--")
        .arg("--include-ignored")
        .assert()
        .success()
        .stdout(predicate::str::contains("✔ ignored test\n"));
}

#[test]
fn cli_reports_ignore_reason() {
    Command::cargo_bin("cargo-testdox")
        .unwrap()
        .current_dir("testdata/testproj")
        .arg("testdox")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "? [expensive test] pi to 1m digits\n",
        ));
}
