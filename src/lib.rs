use anyhow::Context;
use std::process::Command;

#[must_use]
/// Runs `cargo test` with any supplied extra arguments, and returns the
/// resulting standard output.
///
/// # Panics
///
/// If executing the `cargo test` command fails.
pub fn get_cargo_test_output(extra_args: Vec<String>) -> String {
    let mut cargo = Command::new("cargo");
    cargo.arg("test");
    if !extra_args.is_empty() {
        cargo.arg("--");
        cargo.args(extra_args);
    }
    let raw_output = cargo
        .output()
        .context(format!("{cargo:?}"))
        .expect("executing command should succeed")
        .stdout;
    String::from_utf8_lossy(&raw_output).to_string()
}

#[must_use]
pub fn parse_test_results(test_output: &str) -> Vec<TestResult> {
    test_output
        .lines()
        .filter_map(|line| line.strip_prefix("test "))
        .filter(|line| !line.starts_with("result"))
        .map(|line| line.strip_prefix("tests::").unwrap_or(line))
        .map(|line| line.replace('_', " "))
        .map(|line| {
            let splits: Vec<_> = line.split(" ... ").collect();
            let (name, result) = (splits[0], splits[1]);
            TestResult {
                name: name.to_owned(),
                status: match result {
                    "ok" => Status::Pass,
                    "FAILED" => Status::Fail,
                    "ignored" => Status::Ignored,
                    _ => todo!("unhandled test status {:?}", result),
                },
            }
        })
        .collect()
}
pub struct TestResult {
    pub name: String,
    pub status: Status,
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self.status {
            Status::Pass => '✔',
            Status::Fail => 'x',
            Status::Ignored => '?',
        };
        write!(f, " {status} {}", self.name)
    }
}

pub enum Status {
    Pass,
    Fail,
    Ignored,
}
