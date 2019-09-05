# Mini-TT's utilities

[![Crates.io](https://img.shields.io/crates/d/minitt-util.svg)][crates]
[![Crates.io](https://img.shields.io/crates/v/minitt-util.svg)][crates]
[![Crates.io](https://img.shields.io/crates/l/minitt-util.svg)][crates]
[![docs.rs](https://docs.rs/minitt-util/badge.svg)][doc-rs]

 [crates]: https://crates.io/crates/minitt-util/
 [doc-rs]: https://docs.rs/minitt-util

This is a crate extracted from [Mini-TT] to help the development of other
dependently-typed lambda calculus type checkers' command line interface.

It contains helper functions for the [Clap] ([structopt]) command line processor,
file IO, and REPL helpers (for [rustyline]).

As I don't want to break the self-containing property of the [Mini-TT]
codebase, I only extracted things from the CLI helpers, not the type-checker.

 [Mini-TT]: ..
 [Clap]: https://clap.rs
 [structopt]: https://docs.rs/structopt
 [rustyline]: https://docs.rs/rustyline
