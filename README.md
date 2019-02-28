# minitt-rs

[![Crates.io](https://img.shields.io/crates/d/minitt.svg)][crates]
[![Crates.io](https://img.shields.io/crates/v/minitt.svg)][crates]
[![Crates.io](https://img.shields.io/crates/l/minitt.svg)][crates]
[![docs.rs](https://docs.rs/minitt/badge.svg)](https://docs.rs/minitt)
[![Build status][av-svg]][av-url]
[![CircleCI][cc-svg]][cc-url]

 [crates]: https://crates.io/crates/minitt/
 [av-svg]: https://ci.appveyor.com/api/projects/status/0pnq07tqo5skyjeo/branch/master?svg=true
 [av-url]: https://ci.appveyor.com/project/ice1000/minitt-rs/branch/master
 [av-exe]: https://ci.appveyor.com/project/ice1000/minitt-rs/build/artifacts
 [cc-svg]: https://circleci.com/gh/owo-lang/minitt-rs/tree/master.svg?style=svg
 [cc-url]: https://circleci.com/gh/owo-lang/minitt-rs/tree/master

Rust implementation of Mini-TT, a simple dependently-typed lambda calculus.
Built with latest (when developing this) stable Rust, 2018 edition.
It can be used as a core language for complicated dependently-typed programming
languages, or used for testing the correctness of translation algorithms.

I'm trying my best to use complete and meaningful namings.
I'm also doing a general clean-up of the Haskell implementation and comment the
functions with their counterparts' names in the Haskell implementation so people
don't get confused when they read the paper while reading this implementation.

## Resources

+ [Mini-TT Paper](http://www.cse.chalmers.se/~bengt/papers/GKminiTT.pdf)
+ [Code samples](./samples), tested on CI
+ [Doc.rs](https://docs.rs/minitt) documentation

## Features

+ [X] Everything that the Haskell implementation has
+ [X] Parser as a cargo feature
+ [X] AST pretty-printer as a cargo feature
+ Improvements to the original implementation
  + [X] Use `BTreeMap` for branch/case tree so we become flexible on case order
  + [ ] Use `Vec` for telescope instead of functional immutable list
+ New feature apart from trivial improvements
  + [X] Infer type of a pair
  + [ ] Module system (or even a very simple one)
  + (Typed-)Holes?
    + [ ] For completion / context lookup
    + [ ] For type-directed development
+ An executable for CLI usages
  + [X] File checker
  + [ ] REPL
  + [ ] Language server
  + Publish?
    + By `cargo install --path . --bin minittc --all-features`
    + By [AppVeyor][av-exe]
