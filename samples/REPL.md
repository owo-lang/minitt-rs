# REPL Example

This is an example of using the REPL.

```
minitt-rs> minittc --repl
Interactive minittc 0.1.10
Source code: https://github.com/owo-lang/minitt-rs
Issue tracker: https://github.com/owo-lang/minitt-rs/issues/new

The REPL has two modes: the RICH mode and the PLAIN mode.
Completion, history command, hints and (in the future) colored output are available in the rich mode, but does not work entirely under Windows PowerShell ISE and Mintty (Cygwin, MinGW and (possibly, depends on your installation) git-bash).
You are using the RICH mode.

=> :eval 1
1
=> :normalize 1
1
=> :normalize \lambda a. \lambda b. a
λ <0> λ <1> [<0>]
=> :normalize \lambda a. \lambda b. b
λ <0> λ <1> [<1>]
=> :eval \lambda a. \lambda b. a
λ a. λ b. a
=> :gamma
Current Gamma is empty.
=> :load samples\sum-split\bool.minitt
=> :gamma
Current Gamma:
and: Π Sum {False: 1 | True: 1}. _. Π _: bool. bool
bool: Type
boolean_id: Π Sum {False: 1 | True: 1}. _. bool
not: Π Sum {False: 1 | True: 1}. _. bool
=> :context
Current Telescope:
let and: Π _: bool. Π _: bool. bool = split {False λ n. λ _. False n | True λ n. boolean_id};
let boolean_id: Π _: bool. bool = λ n. n;
let not: Π _: bool. bool = split {False λ n. True n | True λ n. False n};
let bool: Type = Sum {False 1 | True 1};

=> :help
Commands:
:quit                Quit the REPL.
:gamma               Show current typing context.
:context             Show current value context.
:load <FILE>         Load an external file.
:infer <EXPR>        Try to infer the type of the given expression.
:type <EXPR>         Try to infer and normalize the type of the given expression.
:eval <EXPR>         Try to evaluate the given expression.
:normalize <EXPR>    Try to evaluate and normalize the type of the given expression.

=> :infer and
Π Sum {False: 1 | True: 1}. _. Π _: bool. bool
=> :infer Type
Type
=> :infer 0
1
=> :infer (0,0)
Σ 1. 1
=> :infer and (False 1)
TypeCheck: Type is not equal to 1 up to normal form
=> let c : Type = 1;
=> :quit
```

