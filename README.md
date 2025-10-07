[![Crate](https://img.shields.io/crates/v/cargo-testdox.svg)](https://crates.io/crates/cargo-testdox)
[![Docs](https://docs.rs/cargo-testdox/badge.svg)](https://docs.rs/cargo-testdox)
![CI](https://github.com/bitfield/cargo-testdox/actions/workflows/ci.yml/badge.svg)
![Audit](https://github.com/bitfield/cargo-testdox/actions/workflows/audit.yml/badge.svg)
![Maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

[Subscribe to learn Rust with me!](https://bitfieldconsulting.com/subscribe)

# cargo-testdox

![cargo-testdox logo](img/logo.png)

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

![Animated demo](img/demo.gif)

`cargo-testdox` will first invoke `cargo test` to run your tests, with any extra arguments that you give it. It will then show the result for each test (passed, failed, or ignored), with the test name formatted as a sentence. That is, with underscores replaced by spaces.

For example, the following test:

```rust,ignore
#[test]
fn it_works() {}
```

will produce this output when run with `cargo-testdox`:

```txt
 ✔ it works
```

If the test were failing, it would produce:

```txt
 x it works
```

If the test were ignored, it would produce:

```txt
 ? it works
```

If the test were in a module `foo::bar`, it would produce:

```txt
 ✔ foo::bar — it works
```

However, if the module path ends with `test` or `tests`, this part is omitted, and the name of the parent module (if there is one) is used instead. For example, if the module is `foo::tests`:

```txt
 ✔ foo — it works
```

Doctests are ignored, since they can't currently be named (pending [RFC #3311](https://github.com/rust-lang/rfcs/pull/3311)).

### Function names with underscores

To avoid underscores in a snake-case function name from being replaced, put `_fn_` after the function name:

```rust,ignore
#[test]
fn print_hello_world_fn_prints_hello_world() {}
```

becomes:

```txt
✔ print_hello_world prints hello world
```

## Why

Because [test names should be sentences](https://bitfieldconsulting.com/posts/test-names).

Compare [`gotestdox`](https://github.com/bitfield/gotestdox), a similar tool for Go tests.

This is an example project from my book [The Secrets of Rust: Tools](https://bitfieldconsulting.com/books/rust-tools).

[![Secrets of Rust: Tools cover image](img/cover.png)](https://bitfieldconsulting.com/books/rust-tools)
