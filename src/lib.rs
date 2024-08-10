//! A Cargo subcommand for printing your Rust test names as sentences.
//!
//! Also contains functions to parse the (human-readable) output of `cargo test`,
//! and to format test names as sentences.
//!
//! Further reading and context: [Test names should be
//! sentences](https://bitfieldconsulting.com/posts/test-names).
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
/// Parses the standard output of `cargo test` into a vec of `TestResult`.
pub fn parse_test_results(test_output: &str) -> Vec<TestResult> {
    test_output.lines().filter_map(parse_line).collect()
}

/// Parses a line from the standard output of `cargo test`.
///
/// If the line represents the result of a test, returns `Some(TestResult)`,
/// otherwise returns `None`.
pub fn parse_line<S: AsRef<str>>(line: S) -> Option<TestResult> {
    let line = line.as_ref().strip_prefix("test ")?;
    if line.starts_with("result") {
        return None;
    }
    let line = line.strip_prefix("tests::").unwrap_or(line);
    let splits: Vec<_> = line.split(" ... ").collect();
    let (name, result) = (splits[0], splits[1]);
    Some(TestResult {
        name: prettify(name),
        status: match result {
            "ok" => Status::Pass,
            "FAILED" => Status::Fail,
            "ignored" => Status::Ignored,
            _ => todo!("unhandled test status {:?}", result),
        },
    })
}

#[must_use]
/// Formats the name of a test function as a sentence.
///
/// Underscores are replaced with spaces.
pub fn prettify<S: AsRef<str>>(input: S) -> String {
    input.as_ref().replace('_', " ")
}

#[derive(Debug, PartialEq)]
/// The (prettified) name and pass/fail status of a given test.
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

#[derive(Debug, PartialEq)]
/// The status of a given test, as reported by `cargo test`.
pub enum Status {
    Pass,
    Fail,
    Ignored,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prettify_returns_expected_results() {
        struct Case {
            input: &'static str,
            want: String,
        }
        let cases = Vec::from([
            Case {
                input: "anagrams_must_use_all_letters_exactly_once",
                want: "anagrams must use all letters exactly once".into(),
            },
            Case {
                input: "no_matches",
                want: "no matches".into(),
            },
            Case {
                input: "single",
                want: "single".into(),
            },
        ]);
        for case in cases {
            assert_eq!(case.want, prettify(case.input));
        }
    }

    #[test]
    fn parse_line_returns_expected_result() {
        struct Case {
            line: &'static str,
            want: Option<TestResult>,
        }
        let cases = Vec::from([Case {
            line: "test foo ... ok",
            want: Some(TestResult {
                name: "foo".into(),
                status: Status::Pass,
            }),
        }]);
        for case in cases {
            assert_eq!(case.want, parse_line(case.line));
        }
    }
}
