use cargo_testdox::{get_cargo_test_output, parse_test_results};

fn main() {
    let output = get_cargo_test_output(std::env::args().skip(1).collect());
    let results = parse_test_results(&output);
    for result in results {
        println!("{result}");
    }
}
