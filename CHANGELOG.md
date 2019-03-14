# Change Log

# 0.2.4

+ `TCS` is refactored to a struct
+ `GenericCaseTree` is refactored to a struct

# 0.2.3

+ Support inferring types of a constructor call
+ Merge `normal.rs` into `read_back.rs`
+ Subtyping: `sum { A }` is a subtype of `sum { A | B }`
+ Add tutorial in `lib.rs`

# 0.2.2

+ Use statically-typed errors instead of strings
+ Add `const` syntactic sugar (#4)
+ Support optional value for sum and split
+ Infer `sum` type

# 0.2.1

+ Make bunch of function public so people can see them in docs.rs
+ Move everything from `type_check.rs` to a `check` module
+ Rename `syntax.rs` to `ast.rs`, `reduce.rs` to `eval.rs`
+ Improve memory efficiency

# 0.2.0

+ REPL support `:eval`, `:infer` and `:normalize`
+ Bunch of improvements on the pretty printer
+ Debugging features of the REPL: `:eval-debug`, `:debug` and `:infer-debug`
+ Make the feature introduced in 0.1.9 correct (#1)

# 0.1.9

+ REPL file completion
+ Support syntax like `let a (b : U): c = bla b;` (#1)
  + Notice: this breaks mutual recursion for some sense
+ Support plain REPL

# 0.1.8

+ Fix the sum type scoping problem mentioned in 0.1.6
+ Improve type-checking error message
+ Add command completion and hint for REPL

# 0.1.7

+ Use `structopt` for command line parsing
+ Initial REPL support
+ Discover that mutual recursion is not working correctly

# 0.1.6

+ Discover and provide a temporary workaround for a scoping issue of sum type
  expressions, maybe fix later
+ Fix badges in `Cargo.toml`
+ Support quiet in cli
+ Support `*` syntactic sugar

# 0.1.5

+ Syntactic sugars
+ Rename `fun` into `split`, let `split` and `sum` use `{ }`
+ Use better variable name in codes, fix some mistakes in comments
+ Fix a very stupid bug. It's too stupid to be written in the changelog.

# 0.1.4

+ Enable parser by default
+ Add executable
+ Add examples
+ Fix bug in scope manipulation
+ Parse patterns

# 0.1.3

+ Simple parser
+ Some fixes in pretty-printer

# 0.1.2

+ Some bug fixes
+ Type inference for pairs

# 0.1.1

+ Support pretty-printer for AST, Normal AST

# 0.1.0

+ Support type-checking

# 0.0.1

+ Create package on https://crates.io
