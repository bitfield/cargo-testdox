[![Crate](https://img.shields.io/crates/v/cargo-testdox.svg)](https://crates.io/crates/cargo-testdox)
[![Docs](https://docs.rs/cargo-testdox/badge.svg)](https://docs.rs/cargo-testdox)
![CI](https://github.com/bitfield/cargo-testdox/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/bitfield/cargo-testdox/actions/workflows/audit.yml/badge.svg)
![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

# cargo-testdox

A Cargo subcommand to print your Rust test names as sentences.

## Installation

```sh
cargo install cargo-testdox
```

## Usage

In any Rust project with tests, run:

```sh
cargo testdox
```

`cargo-testdox` will first invoke `cargo test` to run your tests, with any extra arguments that you give it. It will then show the result for each test (passed, failed, or ignored), with the test name formatted as a sentence. That is, with underscores replaced by spaces.

For example, the following test:

```rust
#[test]
fn it_works() {}
```

will produce this output when run with `cargo-testdox`:

```
 ✔ it works
```

If the test were failing, it would produce:

```
 x it works
```

If the test were ignored, it would produce:

```
 ? it works
```

## Why

Because [test names should be sentences](https://bitfieldconsulting.com/posts/test-names).

Compare [`gotestdox`](https://github.com/bitfield/gotestdox), a similar tool for Go tests.
