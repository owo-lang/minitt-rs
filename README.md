# minitt-rs

![Crates.io](https://img.shields.io/crates/d/minitt.svg)
![Crates.io](https://img.shields.io/crates/v/minitt.svg)
![Crates.io](https://img.shields.io/crates/l/minitt.svg)
[![docs.rs](https://docs.rs/minitt/badge.svg)](https://docs.rs/minitt)
[![Build status](https://ci.appveyor.com/api/projects/status/0pnq07tqo5skyjeo/branch/master?svg=true)](https://ci.appveyor.com/project/ice1000/minitt-rs/branch/master)
[![CircleCI](https://circleci.com/gh/owo-lang/minitt-rs/tree/master.svg?style=svg)](https://circleci.com/gh/owo-lang/minitt-rs/tree/master)

Rust implementation of Mini-TT, a simple dependently-typed lambda calculus.
It can be used as a core language for complicated dependently-typed programming
languages, or used for testing the correctness of translation algorithms.

I'm trying my best to use complete and meaningful namings.
I'm also doing a general clean-up of the Haskell implementation and comment the
functions with their counterparts' names in the Haskell implementation so people
don't get confused when they read the paper while reading this implementation.

[Mini-TT Paper](http://www.cse.chalmers.se/~bengt/papers/GKminiTT.pdf)

### Plan

+ Initial version: just `clone`, `clone`, `clone` and simulate the Haskell implementation
+ Update: use mutable `Vec`/`BTreeMap` for telescopes instead of functional immutable lists to improve efficiency
+ Update: replace the use of `Clone`/`Box` to the use of `Rc`
+ Update: add a parser, enabled with `feature`
