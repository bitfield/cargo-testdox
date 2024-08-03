use std::process::Command;

fn main() {
    let mut cargo_test = Command::new("cargo");
    cargo_test.arg("test");
    if std::env::args().len() > 1 {
        cargo_test.arg("--");
        cargo_test.args(std::env::args().skip(1));
    }
    let raw_output = cargo_test.output()
    .expect("failed to execute process")
    .stdout;
    let output = String::from_utf8_lossy(&raw_output);
    let results: Vec<String> = output.lines()
    .filter_map(|line| line.strip_prefix("test "))
    .filter(|line| !line.starts_with("result"))
    .map(|line| line.strip_prefix("tests::").unwrap_or(line))
    .map(|line| line.replace('_', " "))
    .collect();
    for result in results {
        println!("{result}");
    }
}
