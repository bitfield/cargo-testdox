#[cfg(test)]
mod tests {
    #[test]
    fn open_file_fn_returns_file_handle_on_success() {
        assert_eq!(1 + 1, 2);
    }

    fn open_file_fn_returns_error_on_failure() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    fn add_returns_2_for_1_plus_1() {
        assert_eq!(3, 2);
    }

    #[test]
    #[ignore]
    fn ignored_test() {
        assert_eq!(1 + 1, 2);
    }
}

#[cfg(test)]
mod fancy_module {
    #[test]
    fn fancy_function_fn_returns_right_answer() {
        assert_eq!(1 + 1, 2);
    }
}
