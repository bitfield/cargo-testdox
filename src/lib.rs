#![doc = include_str!("../README.md")]
use anyhow::{anyhow, Context};
use colored::Colorize;
use std::{fmt::Display, process::Command, str::FromStr};

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
    cargo.args(extra_args);
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
pub fn parse_line(line: impl AsRef<str>) -> Option<TestResult> {
    let line = line.as_ref().strip_prefix("test ")?;
    if line.starts_with("result") || line.contains("(line ") {
        return None;
    }

    let (test, status) = line.split_once(" ... ")?;
    let (module, name) = match test.rsplit_once("::") {
        Some((module, name)) => (prettify_module(module), name),
        None => (None, test),
    };
    Some(TestResult {
        module,
        name: prettify(name),
        status: status.parse().ok()?,
    })
}

#[must_use]
/// Formats the name of a test function as a sentence.
///
/// Underscores are replaced with spaces. To retain the underscores in a function name, put `_fn_` after it. For example:
///
/// ```text
/// parse_line_fn_parses_a_line
/// ```
///
/// becomes:
///
/// ```text
/// parse_line parses a line
/// ```
pub fn prettify(input: impl AsRef<str>) -> String {
    if let Some((fn_name, sentence)) = input.as_ref().split_once("_fn_") {
        format!("{} {}", fn_name, humanize(sentence))
    } else {
        humanize(input)
    }
}

fn humanize(input: impl AsRef<str>) -> String {
    input
        .as_ref()
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

fn prettify_module(module: &str) -> Option<String> {
    let mut parts = module.split("::").collect::<Vec<_>>();
    parts.pop_if(|&mut s| s == "tests" || s == "test");
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("::"))
}

#[derive(Debug, PartialEq)]
/// The (prettified) name and pass/fail status of a given test.
pub struct TestResult {
    pub module: Option<String>,
    pub name: String,
    pub status: Status,
}

impl Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.module {
            Some(module) => write!(
                f,
                "{} {} – {}",
                self.status,
                module.bright_blue(),
                self.name
            ),
            None => write!(f, "{} {}", self.status, self.name),
        }
    }
}

#[derive(Debug, PartialEq)]
/// The status of a given test, as reported by `cargo test`.
pub enum Status {
    Pass,
    Fail,
    Ignored,
}

impl FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(status: &str) -> Result<Self, Self::Err> {
        match status {
            "ok" => Ok(Status::Pass),
            "FAILED" => Ok(Status::Fail),
            "ignored" => Ok(Status::Ignored),
            _ => Err(anyhow!("unhandled test status {status:?}")),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            Status::Pass => "✔".bright_green(),
            Status::Fail => "x".bright_red(),
            Status::Ignored => "?".bright_yellow(),
        };
        write!(f, "{status}")
    }
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
            Case {
                input: "parse_line_fn_does_stuff",
                want: "parse_line does stuff".into(),
            },
            Case {
                input: "prettify__handles_multiple_underscores",
                want: "prettify handles multiple underscores".into(),
            },
            Case {
                input: "prettify_fn__handles_multiple_underscores",
                want: "prettify handles multiple underscores".into(),
            },
        ]);
        for case in cases {
            assert_eq!(case.want, prettify(case.input));
        }
    }

    #[test]
    fn parse_line_fn_returns_expected_result() {
        struct Case {
            line: &'static str,
            want: Option<TestResult>,
        }
        let cases = Vec::from([
            Case {
                line: "    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s",
                want: None,
            },
            Case {
                line: "test foo ... ok",
                want: Some(TestResult {
                    module: None,
                    name: "foo".into(),
                    status: Status::Pass,
                }),
            },
            Case {
                line: "test foo::tests::does_foo_stuff ... ok",
                want: Some(TestResult {
                    module: Some("foo".into()),
                    name: "does foo stuff".into(),
                    status: Status::Pass,
                }),
            },
            Case {
                line: "test tests::urls_correctly_extracts_valid_urls ... FAILED",
                want: Some(TestResult {
                    module: None,
                    name: "urls correctly extracts valid urls".into(),
                    status: Status::Fail,
                }),
            },
            Case {
                line: "test files::test::files_can_be_sorted_in_descending_order ... ignored",
                want: Some(TestResult {
                    module: Some("files".into()),
                    name: "files can be sorted in descending order".into(),
                    status: Status::Ignored,
                }),
            },
            Case {
                line: "test files::test::foo::tests::files_can_be_sorted_in_descending_order ... ignored",
                want: Some(TestResult {
                    module: Some("files::test::foo".into()),
                    name: "files can be sorted in descending order".into(),
                    status: Status::Ignored,
                }),
            },
            Case {
                line: "test files::test_foo::files_can_be_sorted_in_descending_order ... ignored",
                want: Some(TestResult {
                    module: Some("files::test_foo".into()),
                    name: "files can be sorted in descending order".into(),
                    status: Status::Ignored,
                }),
            },
            Case {
                line: "test src/lib.rs - find_top_n_largest_files (line 17) ... ok",
                want: None,
            },
            Case {
                line: "test output_format::_concise_expects ... ok",
                want: Some(TestResult {
                    module: Some("output_format".into()),
                    name: "concise expects".into(),
                    status: Status::Pass,
                }),
            },
        ]);
        for case in cases {
            assert_eq!(case.want, parse_line(case.line));
        }
    }
}
