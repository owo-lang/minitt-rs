# minitt-rs

[![Crates.io](https://img.shields.io/crates/d/minitt.svg)][crates]
[![Crates.io](https://img.shields.io/crates/v/minitt.svg)][lib-rs]
[![Crates.io](https://img.shields.io/crates/l/minitt.svg)][crates]
[![docs.rs](https://docs.rs/minitt/badge.svg)][doc-rs]
[![Actions Status][ga-svg]][ga-url]
[![dependency status][dep-svg]][dep-rs]

 [crates]: https://crates.io/crates/minitt/
 [lib-rs]: https://lib.rs/minitt/
 [doc-rs]: https://docs.rs/minitt
 [dep-rs]: https://deps.rs/repo/github/owo-lang/minitt-rs
 [dep-svg]: https://deps.rs/repo/github/owo-lang/minitt-rs/status.svg
 [clap]: https://clap.rs/
 [pest]: https://pest.rs/
 [rustyline]: https://docs.rs/rustyline/
 [icon]: https://raw.githubusercontent.com/owo-lang/minitt-rs/master/rustdoc/icon.svg?sanitize=true
 [plugin]: https://github.com/owo-lang/intellij-dtlc/
 [voile]: https://github.com/owo-lang/voile-rs
 [ga-svg]: https://github.com/owo-lang/minitt-rs/workflows/build/badge.svg
 [ga-url]: https://github.com/owo-lang/minitt-rs/actions

Rust implementation of Mini-TT, a simple dependently-typed lambda calculus.
This implementation includes a type-checker (extended the origin), an AST
pretty-printer and a command line tool which can be used as a file checker and
an interactive REPL with completion and type inference.
Built with stable Rust (version 1.39.0), 2018 edition.
It can be used as a core language for complicated dependently-typed programming
languages, or used for testing the correctness of translation algorithms.

I'm trying my best to use complete and meaningful naming to avoid confusion.
I'm also doing a general clean-up of the Haskell implementation and comment the
functions with their counterparts' names in the Haskell implementation so people
don't get confused when they read the paper while reading this implementation.

*Notice: the development of this POC language has been moved to another
redesigned programming language, [Voile][voile]. We have a new type theory,
better surface syntax, better error messages, richer type-checker. Everything
is (or will become) better in Voile.*

A dependently-typed program in [samples](../samples/dependent/function.minitt):

```haskell
-- A 2 type and a 1 type
const bool = Sum { True | False };
const unit = Sum { TT };

-- By `function.minitt` of course I mean dependent functions :)
let return_type: bool -> Type = split
 { True => unit
 | False => 1
 };

-- Return things that are of different types.
let function: \Pi b: bool. return_type b = split
 { True => TT
 | False => 0
 };
```

We can have functions returning values of different types, while it's still
statically-typed. Very flexible.

## Install

The most recommended way of installation is to download the prebuilt binaries
from [GitHub Actions page][ga-url].
Here's [how to find them](https://github.com/actions/upload-artifact).

You may also install from source:

```bash
$ cargo install minitt --bin minittc --all-features --force
```

Want to use minitt as a library? Add this to your `Cargo.toml`
(if you don't even need a parser, you can remove the features completely):

```toml
minitt = { version = "0.4.2", features = ["parser"] }
```

## Resources

![][icon]

+ [Mini-TT Paper](http://www.cse.chalmers.se/~bengt/papers/GKminiTT.pdf)
+ [Code Samples](../samples), tested on CI
+ [Doc.rs][doc-rs] documentation (a tutorial is included)
+ [Change Log](../CHANGELOG.md)
+ [REPL Example](../samples/REPL.md)
+ [**Binary Download**][ga-url] on GitHub Actions page for
  Windows, Ubuntu and macOS
+ [IntelliJ Plugin][plugin], as a part of project Dependently-Typed Lambda
  Calculus
+ [Mini-TT Utilities](../minitt-util) helper library for making a CLI REPL

## Features

+ [X] Everything that the Haskell implementation has
+ [X] Parser as a cargo feature (using [pest][pest])
+ [X] AST pretty-printer as a cargo feature
+ [ ] Improvements to the original implementation
  + [X] Use `BTreeMap` for branch/case tree so we become flexible on case order
  + [ ] Use `Vec` for telescope/declaration instead of functional immutable list
+ [ ] New feature apart from trivial improvements
  + [X] Infer type of a pair
  + [X] Infer type of a constructor call
  + [ ] Infer type of a case-split
  + [ ] Module system (or even a very simple one)
  + [ ] (Typed-)Holes?
    + [ ] For completion / context lookup
    + [ ] For type-directed development
  + [X] `const` declarations, where the type is inferred
  + [X] Prefixing declarations with parameters, like `let a (b: c): d = f b;`
+ [ ] An executable for CLI usages (`minittc`) (using [clap][clap])
  + [X] File checker
  + [X] Completion script generation
    + Get the script: `minittc completion zsh/bash/powershell/fish/elvish`
  + [X] REPL (a fancy one based on [rustyline][rustyline] and a plain
    one based on stdio)
    + [X] Load file
    + [X] Infer (and normalize) type
    + [X] Eval (and normalize) expressions (may panic if ill-typed)
    + [X] Add single declaration
    + [X] Show context/gamma
    + [X] Help
    + [X] Completion
      + [X] Commands
      + [X] Files
      + [ ] In-scope variables
  + [ ] Language server (?)
    + We've got an [IntelliJ plugin][plugin]
  + [X] Publish?
    + By `cargo install --path . --bin minittc --all-features`
    + By [AppVeyor][av-url]
      (Just find the configuration fits you best and get the artifact)
