use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn passing_bins_flag_runs_binary_tests() {
    let temp_dir = TempDir::new().unwrap();
    create_test_project(&temp_dir).unwrap();

    Command::cargo_bin("cargo-testdox")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("testdox")
        .arg("--bins")
        .assert()
        .success()
        .stdout(predicate::str::contains("test greet"))
        .stdout(predicate::str::contains("test one").not())
        .stdout(predicate::str::contains("test two").not())
        .stdout(predicate::function(|output: &str| {
            output.matches("✔").count() == 1
        }));
}

#[test]
fn passing_lib_flag_runs_library_tests() {
    let temp_dir = TempDir::new().unwrap();
    create_test_project(&temp_dir).unwrap();

    Command::cargo_bin("cargo-testdox")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("testdox")
        .arg("--lib")
        .assert()
        .success()
        .stdout(predicate::str::contains("test one"))
        .stdout(predicate::str::contains("test two"))
        .stdout(predicate::str::contains("test greet").not())
        .stdout(predicate::function(|output: &str| {
            output.matches("✔").count() == 2
        }));
}

#[test]
fn passing_include_ignored_flag_runs_ignored_tests() {
    let temp_dir = TempDir::new().unwrap();
    create_test_project(&temp_dir).unwrap();

    Command::cargo_bin("cargo-testdox")
        .unwrap()
        .current_dir(&temp_dir)
        .arg("testdox")
        .arg("--lib")
        .arg("--")
        .arg("--include-ignored")
        .assert()
        .success()
        .stdout(predicate::str::contains("test one"))
        .stdout(predicate::str::contains("test two"))
        .stdout(predicate::str::contains("ignored test one"))
        .stdout(predicate::str::contains("ignored test two"))
        .stdout(predicate::function(|output: &str| {
            output.matches("✔").count() == 4
        }));
}

fn create_test_project(dir: &TempDir) -> std::io::Result<()> {
    let cargo_toml = r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2024"
"#;
    std::fs::create_dir(dir.path().join("src"))?;
    std::fs::write(dir.path().join("Cargo.toml"), cargo_toml)?;

    let lib_rs = r"
#[cfg(test)]
mod tests {
    #[test]
    fn test_one() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn test_two() {
        assert_eq!(2 * 2, 4);
    }

    #[test]
    #[ignore]
    fn ignored_test_one() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    #[ignore]
    fn ignored_test_two() {
        assert_eq!(2 * 2, 4);
    }
}";
    std::fs::write(dir.path().join("src").join("lib.rs"), lib_rs)?;

    let main_rs = r#"
fn main() {
    println!("{}", greet("World"));
}

fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("Alice"), "Hello, Alice!");
        assert_eq!(greet("Bob"), "Hello, Bob!");
   }
}
"#;
    std::fs::write(dir.path().join("src").join("main.rs"), main_rs)?;

    Ok(())
}
