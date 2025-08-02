//! A Cargo subcommand for printing your Rust test names as sentences.
//!
//! Also contains functions to parse the (human-readable) output of `cargo test`,
//! and to format test names as sentences.
//!
//! Further reading and context: [Test names should be
//! sentences](https://bitfieldconsulting.com/posts/test-names).
use anyhow::Context;
use colored::Colorize;
use regex::Regex;
use std::{process::Command, sync::LazyLock};

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

static MODULE_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^((?:\w+::)+)").unwrap());

/// Parses a line from the standard output of `cargo test`.
///
/// If the line represents the result of a test, returns `Some(TestResult)`,
/// otherwise returns `None`.
pub fn parse_line<S: AsRef<str>>(line: S) -> Option<TestResult> {
    let line = line.as_ref().strip_prefix("test ")?;
    if line.starts_with("result") || line.contains("(line ") {
        return None;
    }

    let (module, line) = match MODULE_PREFIX.captures(line) {
        Some(captures) => {
            let prefix = &captures[1];
            let module = prefix.strip_suffix("::")?;
            let line = line.strip_prefix(prefix)?;
            (Some(module.to_string()), line)
        }
        None => (None, line),
    };

    let splits: Vec<_> = line.split(" ... ").collect();
    let (name, result) = (splits[0], splits[1]);
    Some(TestResult {
        module,
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
pub fn prettify<S: AsRef<str>>(input: S) -> String {
    let mut output = String::new();
    if let Some((fn_name, sentence)) = input.as_ref().split_once("_fn_") {
        output.push_str(fn_name);
        output.push(' ');
        output.push_str(sentence.replace('_', " ").as_ref());
    } else {
        output.push_str(input.as_ref().replace('_', " ").as_ref());
    }
    output
}

#[derive(Debug, PartialEq)]
/// The (prettified) name and pass/fail status of a given test.
pub struct TestResult {
    pub module: Option<String>,
    pub name: String,
    pub status: Status,
}

impl std::fmt::Display for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.module {
            Some(module) => write!(f, "{} {module} {}", self.status, self.name),
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

impl std::fmt::Display for Status {
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
                line: "test tests::urls_correctly_extracts_valid_urls ... FAILED",
                want: Some(TestResult {
                    module: Some("tests".into()),
                    name: "urls correctly extracts valid urls".into(),
                    status: Status::Fail,
                }),
            },
            Case {
                line: "test files::test::files_can_be_sorted_in_descending_order ... ignored",
                want: Some(TestResult {
                    module: Some("files::test".into()),
                    name: "files can be sorted in descending order".into(),
                    status: Status::Ignored,
                }),
            },
            Case {
                line: "test src/lib.rs - find_top_n_largest_files (line 17) ... ok",
                want: None,
            },
        ]);
        for case in cases {
            assert_eq!(case.want, parse_line(case.line));
        }
    }
}
