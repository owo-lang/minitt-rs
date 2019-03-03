# REPL Example

This is an example of using the REPL.

```
minitt-rs> minittc --repl
=> :g
Current Gamma is empty.
=> :l samples\sum-split\bool.minitt
=> :g
Current Gamma:
and: Π sum {False: 1 | True: 1}. _. Π _: bool. bool
bool: U
boolean_id: Π sum {False: 1 | True: 1}. _. bool
not: Π sum {False: 1 | True: 1}. _. bool
=> :c
Current Telescope:
let and: Π _: bool. Π _: bool. bool = split {False λ n. λ _. False n | True λ n. boolean_id};
let boolean_id: Π _: bool. bool = λ n. n;
let not: Π _: bool. bool = split {False λ n. True n | True λ n. False n};
let bool: U = sum {False 1 | True 1};

=> :h
Interactive minittc 0.1.7
Commands:
:quit    :q        Quit the REPL.
:gamma   :g        Show current typing context.
:context :c        Show current value context.
:load    :l <FILE> Load an external file.
:type    :t <EXPR> Try to infer the type of an expression.

=> :t and
Π sum {False: 1 | True: 1}. _. Π _: bool. bool
=> :t U
U
=> :t 0
1
=> :t (0,0)
Σ 1. 1
=> :t and (False 1)
TypeCheck: U is not equal to 1 up to normal form
=> let c : U = 1;
=> :q
```

