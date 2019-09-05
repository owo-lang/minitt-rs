/*!
# Mini-TT Util

This is a crate extracted from the codebase of the [Mini-TT] type checker
to help the development of other dependently-typed lambda calculus type checkers'
command line interface.

It contains helper functions for the [Clap] ([structopt]) command line processor,
file IO, and REPL helpers (for [rustyline]).

As I don't want to break the self-containing property of the [Mini-TT]
codebase, I only extracted things from the CLI helpers, not the type-checker.

All dependencies are optional, thus very lightweight.

 [Mini-TT]: https://docs.rs/minitt
 [Clap]: https://clap.rs
 [structopt]: https://docs.rs/structopt
 [rustyline]: https://docs.rs/rustyline
*/

/// For Command-line processing, etc.
#[cfg(feature = "cli")]
pub mod cli;

/// File IO.
pub mod io;

/// For REPL.
#[cfg(feature = "repl")]
pub mod repl;
