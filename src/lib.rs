#![doc = include_str!("../README.md")]
use anyhow::anyhow;
use colored::Colorize;
use std::io::{self, Read};
use std::{fmt::Display, process::Command, str::FromStr};

#[must_use]
/// Runs `cargo test` with any supplied extra arguments, and returns the
/// resulting standard output.
///
/// # Panics
///
/// If executing the `cargo test` command fails.
pub fn get_cargo_test_output(extra_args: Vec<String>) -> String {
    let (mut reader, writer) = io::pipe().expect("failed to build pipe");

    let mut cargo = Command::new("cargo")
        .arg("test")
        .args(extra_args)
        .stdout(writer.try_clone().expect("failed to clone pipe"))
        .stderr(writer)
        .spawn()
        .expect("failed to execute child");

    let mut output = String::new();
    reader
        .read_to_string(&mut output)
        .expect("failed to read output");

    cargo.wait().expect("failed to execute command");

    output
}

#[must_use]
/// Parses the standard output of `cargo test` into a vec of `TestGroup`.
///
/// Results are returned sorted by module, name, and status for better readability.
pub fn parse_test_results(test_output: &str) -> Vec<TestGroup> {
    let lines: Vec<Line> = test_output.lines().filter_map(parse_line).collect();

    let mut test_groups = vec![];
    let mut index = 0;
    for line in lines {
        match line {
            Line::Location { path, test_type } => {
                let test_group = TestGroup::new(path, test_type);
                test_groups.push(test_group);
                index = test_groups.len() - 1;
            }
            Line::Test(test_result) => {
                test_groups[index].tests.push(test_result);
            }
        }
    }

    for group in &mut test_groups {
        group.tests.sort();
    }

    test_groups
}

#[derive(Debug, PartialEq)]
/// Indicates type of tests - unit or integration.
pub enum TestType {
    // unit test
    Unit,
    // integration test
    Integration,
}

#[derive(Debug, PartialEq)]
/// Type of parsed line - test or location of tests suite.
pub enum Line {
    // represents a parsed line with test info
    Test(TestResult),
    // represents a parsed line with tests suite info
    Location { path: String, test_type: TestType },
}

/// Parses a line from the standard output of `cargo test`.
///
/// If the line represents the result of a test or location, returns `Some(Line)`,
/// otherwise returns `None`.
pub fn parse_line(line: impl AsRef<str>) -> Option<Line> {
    let line = &line.as_ref().trim_start();

    if line.starts_with("Running unittests") {
        let ar: Vec<&str> = line.split(" ").collect();
        let mut iter = ar.iter();
        iter.next();
        iter.next();
        let path = iter.next().expect("path to be a third item");
        return Some(Line::Location {
            path: path.to_string(),
            test_type: TestType::Unit,
        });
    }

    if line.starts_with("Running") {
        let ar: Vec<&str> = line.split(" ").collect();
        let mut iter = ar.iter();
        iter.next();
        let path = iter.next().expect("path to be a second item");
        return Some(Line::Location {
            path: path.to_string(),
            test_type: TestType::Integration,
        });
    }

    let line = line.strip_prefix("test ")?;
    if line.starts_with("result") || line.contains("(line ") {
        return None;
    }

    let (test, status) = line.split_once(" ... ")?;
    let (module, name) = match test.rsplit_once("::") {
        Some((module, name)) => (prettify_module(module), name),
        None => (None, test),
    };

    Some(Line::Test(TestResult {
        module,
        name: prettify(name),
        status: status.parse().ok()?,
    }))
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

#[derive(Debug)]
/// Serves for grouping test in one structure with location of tests suite and tests type (unit or
/// integration).
pub struct TestGroup {
    // location of tests suite
    loc: String,
    #[allow(unused)]
    // tests type - unit or integration
    test_type: TestType,
    // test results
    tests: Vec<TestResult>,
}

impl TestGroup {
    /// Builds new Test group with given location and tests type (unit or integration).
    pub fn new(loc: String, test_type: TestType) -> Self {
        Self {
            loc,
            test_type,
            tests: vec![],
        }
    }

    /// Returns test results.
    pub fn tests(&self) -> &Vec<TestResult> {
        &self.tests
    }
}

impl Display for TestGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.loc)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// The status of a given test, as reported by `cargo test`.
pub enum Status {
    Pass,
    Fail,
    Ignored(Option<String>),
}

impl FromStr for Status {
    type Err = anyhow::Error;

    fn from_str(status: &str) -> Result<Self, Self::Err> {
        match status {
            "ok" => Ok(Status::Pass),
            "FAILED" => Ok(Status::Fail),
            "ignored" => Ok(Status::Ignored(None)),
            _ => {
                if let Some((_, reason)) = status.split_once(", ") {
                    Ok(Status::Ignored(Some(reason.to_string())))
                } else {
                    Err(anyhow!("unhandled test status {status:?}"))
                }
            }
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            Status::Pass => "✔".bright_green(),
            Status::Fail => "x".bright_red(),
            Status::Ignored(None) => "?".bright_yellow(),
            Status::Ignored(Some(reason)) => format!("? [{reason}]").bright_yellow(),
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
            want: Option<Line>,
        }
        let cases = Vec::from([
            Case {
                line: "    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s",
                want: None,
            },
            Case {
                line: "test foo ... ok",
                want: Some(Line::Test(TestResult {
                    module: None,
                    name: "foo".into(),
                    status: Status::Pass,
                })),
            },
            Case {
                line: "test foo::tests::does_foo_stuff ... ok",
                want: Some(Line::Test(TestResult {
                    module: Some("foo".into()),
                    name: "does foo stuff".into(),
                    status: Status::Pass,
                })),
            },
            Case {
                line: "test tests::urls_correctly_extracts_valid_urls ... FAILED",
                want: Some(Line::Test(TestResult {
                    module: None,
                    name: "urls correctly extracts valid urls".into(),
                    status: Status::Fail,
                })),
            },
            Case {
                line: "test files::test::files_can_be_sorted_in_descending_order ... ignored",
                want: Some(Line::Test(TestResult {
                    module: Some("files".into()),
                    name: "files can be sorted in descending order".into(),
                    status: Status::Ignored(None),
                })),
            },
            Case {
                line: "test files::test::foo::tests::files_can_be_sorted_in_descending_order ... ignored",
                want: Some(Line::Test(TestResult {
                    module: Some("files::test::foo".into()),
                    name: "files can be sorted in descending order".into(),
                    status: Status::Ignored(None),
                })),
            },
            Case {
                line: "test files::test_foo::files_can_be_sorted_in_descending_order ... ignored",
                want: Some(Line::Test(TestResult {
                    module: Some("files::test_foo".into()),
                    name: "files can be sorted in descending order".into(),
                    status: Status::Ignored(None),
                })),
            },
            Case {
                line: "test tests::pi_to_1m_digits ... ignored, expensive test",
                want: Some(Line::Test(TestResult {
                    module: None,
                    name: "pi to 1m digits".into(),
                    status: Status::Ignored(Some("expensive test".into())),
                })),
            },
            Case {
                line: "test src/lib.rs - find_top_n_largest_files (line 17) ... ok",
                want: None,
            },
            Case {
                line: "test output_format::_concise_expects ... ok",
                want: Some(Line::Test(TestResult {
                    module: Some("output_format".into()),
                    name: "concise expects".into(),
                    status: Status::Pass,
                })),
            },
            Case {
                line: "Running tests/user.rs (/full/path/to/test)",
                want: Some(Line::Location { path: String::from("tests/user.rs"), test_type: TestType::Integration }),
            },
            Case {
                line: "Running unittests src/main.rs (/full/path/to/test)",
                want: Some(Line::Location { path: String::from("src/main.rs"), test_type: TestType::Unit }),
            },
        ]);
        for case in cases {
            assert_eq!(case.want, parse_line(case.line));
        }
    }

    // test results sort by module, name, and status.
    #[test]
    fn test_results_sort_by_module_name_and_status() {
        let mut results = vec![
            TestResult {
                module: Some("zeta".into()),
                name: "zebra".into(),
                status: Status::Pass,
            },
            TestResult {
                module: None,
                name: "alpha".into(),
                status: Status::Fail,
            },
            TestResult {
                module: None,
                name: "alpha".into(),
                status: Status::Pass,
            },
            TestResult {
                module: Some("alpha".into()),
                name: "beta".into(),
                status: Status::Ignored(None),
            },
            TestResult {
                module: Some("alpha".into()),
                name: "alpha".into(),
                status: Status::Pass,
            },
        ];
        results.sort();
        let expected = vec![
            TestResult {
                module: None,
                name: "alpha".into(),
                status: Status::Pass,
            },
            TestResult {
                module: None,
                name: "alpha".into(),
                status: Status::Fail,
            },
            TestResult {
                module: Some("alpha".into()),
                name: "alpha".into(),
                status: Status::Pass,
            },
            TestResult {
                module: Some("alpha".into()),
                name: "beta".into(),
                status: Status::Ignored(None),
            },
            TestResult {
                module: Some("zeta".into()),
                name: "zebra".into(),
                status: Status::Pass,
            },
        ];
        assert_eq!(results, expected, "wrong order");
    }
}
