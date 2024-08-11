use cargo_testdox::{get_cargo_test_output, parse_test_results, Status};

fn main() {
    let output = get_cargo_test_output(std::env::args().skip(2).collect());
    let results = parse_test_results(&output);
    let mut failed = false;
    for result in results {
        println!("{result}");
        if result.status == Status::Fail {
            failed = true;
        }
    }
    if failed {
        std::process::exit(1);
    }
}
