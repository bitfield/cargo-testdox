use cargo_testdox::{get_cargo_test_output, parse_test_results, Status};

fn main() {
    let output = get_cargo_test_output(std::env::args().skip(2).collect());
    let test_groups = parse_test_results(&output);

    let mut failed = false;
    for group in test_groups {
        let tests = group.tests();

        if !tests.is_empty() {
            println!("{group}");
        }
        for test in tests {
            println!("{test}");
            if test.status == Status::Fail {
                failed = true;
            }
        }
    }
    if failed {
        std::process::exit(1);
    }
}
