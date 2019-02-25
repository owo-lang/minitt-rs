# minitt-rs

Rust implementation of Mini-TT, a simple dependently-typed lambda calculus.
It can be used as a core language for complicated dependently-typed programming
languages, or used for testing the correctness of translation algorithms.

I'm trying my best to use complete and meaningful namings.

[Mini-TT Paper](http://www.cse.chalmers.se/~bengt/papers/GKminiTT.pdf)

### Plan

+ Initial version: just `clone`, `clone`, `clone` and simulate the Haskell implementation
+ Update: use mutable `Vec`/`BTreeMap` to improve efficiency
+ Update: replace the use of `Clone`/`Box` to the use of `Rc`
+ Update: add a parser, enabled with `feature`
