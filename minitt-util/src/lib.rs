/*!
# Mini-TT Util

This is a crate extracted from the codebase of the [Mini-TT] type checker
to help the development of other dependently-typed lambda calculus type checkers'
command line interface.

It contains helper functions for the [Clap] ([structopt]) command line processor.

As I don't want to break the self-containing property of the [Mini-TT]
codebase, I only extracted things from the CLI helpers, not the type-checker.

 [Mini-TT]: https://docs.rs/minitt
 [Clap]: https://clap.rs
 [structopt]: https://docs.rs/structopt
*/
