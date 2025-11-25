#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn ignored_test() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    #[ignore = "expensive test"]
    fn pi_to_1m_digits() {}
}
