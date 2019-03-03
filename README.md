# minitt-rs

[![Crates.io](https://img.shields.io/crates/d/minitt.svg)][crates]
[![Crates.io](https://img.shields.io/crates/v/minitt.svg)][crates]
[![Crates.io](https://img.shields.io/crates/l/minitt.svg)][crates]
[![docs.rs](https://docs.rs/minitt/badge.svg)][doc-rs]
[![Build status][av-svg]][av-url]
[![CircleCI][cc-svg]][cc-url]
[![dependency status][dep-svg]][dep-rs]

 [crates]: https://crates.io/crates/minitt/
 [av-svg]: https://ci.appveyor.com/api/projects/status/0pnq07tqo5skyjeo/branch/master?svg=true
 [av-url]: https://ci.appveyor.com/project/ice1000/minitt-rs/branch/master
 [cc-svg]: https://circleci.com/gh/owo-lang/minitt-rs/tree/master.svg?style=svg
 [cc-url]: https://circleci.com/gh/owo-lang/minitt-rs/tree/master
 [doc-rs]: https://docs.rs/minitt
 [dep-rs]: https://deps.rs/repo/github/owo-lang/minitt-rs
 [dep-svg]: https://deps.rs/repo/github/owo-lang/minitt-rs/status.svg
 [clap]: https://clap.rs/

Rust implementation of Mini-TT, a simple dependently-typed lambda calculus.
Built with latest (when developing this) stable Rust, 2018 edition.
It can be used as a core language for complicated dependently-typed programming
languages, or used for testing the correctness of translation algorithms.

I'm trying my best to use complete and meaningful namings.
I'm also doing a general clean-up of the Haskell implementation and comment the
functions with their counterparts' names in the Haskell implementation so people
don't get confused when they read the paper while reading this implementation.

A dependently-typed program in [samples](./samples/dependent/function.minitt):

```haskell
let bool: U = sum { True 1 | False 1 };
let unit: U = sum { TT 1 };
-- A 2 type and a 1 type

let return_type: bool -> U = split
 { True _ => unit
 | False _ => 1
 };
-- By `function.minitt` of course I mean dependent functions :)

let function: \Pi b: bool. return_type b = split
 { True _ => TT 0
 | False _ => 0
 };
-- Return things that are of different types.
```

We can have functions returning values of different types, while it's still
statically-typed. Very flexible.

## Resources

+ [Mini-TT Paper](http://www.cse.chalmers.se/~bengt/papers/GKminiTT.pdf)
+ [Code samples](./samples), tested on CI
+ [Doc.rs][doc-rs] documentation
+ [Change log](CHANGELOG.md)
+ Windows binary download by [AppVeyor][av-url]

## Features

+ [X] Everything that the Haskell implementation has
+ [X] Parser as a cargo feature
+ [X] AST pretty-printer as a cargo feature
+ Improvements to the original implementation
  + [ ] Fix recursive data type definition scoping problem
  + [X] Use `BTreeMap` for branch/case tree so we become flexible on case order
  + [ ] Use `Vec` for telescope/declaration instead of functional immutable list
+ New feature apart from trivial improvements
  + [X] Infer type of a pair
  + [ ] Infer type of a constructor call
  + [ ] Infer type of a case-split
  + [ ] Module system (or even a very simple one)
  + (Typed-)Holes?
    + [ ] For completion / context lookup
    + [ ] For type-directed development
+ An executable for CLI usages (`minittc`)
  + [X] File checker
  + [X] Completion script generation
    + Get the script: `minittc completion zsh/bash/powershell/fish/elvish`
    + Thanks to [clap][clap]!
  + REPL
    + [X] Load file
    + [X] Infer type
    + [X] Add single declaration
    + [X] Show context/gamma
    + [X] Help
    + [ ] Completion
  + [ ] Language server
  + Publish?
    + By `cargo install --path . --bin minittc --all-features`
    + By [AppVeyor][av-url]
      (Just find the configuration fits you best and get the artifact)
