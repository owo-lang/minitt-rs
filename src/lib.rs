/*!
Reading the [README](https://github.com/owo-lang/minitt-rs/blob/master/README.md) is recommended
before reading the documentation.

# Tutorial

Here's a brief summary of the complete type-checking process.

Since this implementation is actually a dialect or, a variation of the original one, I use *minitt*
to represent this implementation and *Mini-TT* for the original one.

Here's a "feature list" (only language features that affect type-checking are listed):

First, Mini-TT supports:

+ Pi/Sigma types
+ First-class sum types and case-split
+ Recursion
+ Mutual recursion

Mini-TT does not support (while you may expect it to support):

+ Dependent pattern matching (with unification)
+ Meta variables, say, implicit arguments

Mini-TT does not, but minitt does support:

+ Functions returning functions (curried functions) (cubicaltt supports this too) with the
  help of an [additional member of lambda expressions](ast/struct.AnonymousValue.html)
+ Infer types of expressions that appears deeply inside an expression
+ Constant expressions with type signature completely inferred
+ Universe levels and subtyping (work in progress)
+ Subtyping on sum types (work in progress)

## Syntax Trees

Mini-TT has three syntax trees:

+ [Surface syntax tree](ast/enum.Expression.html), aka concrete syntax tree, representing
  expressions that cannot be type-checked alone or simply not type-checked yet
+ [Abstract syntax tree](ast/enum.Value.html), aka values or terms, representing expressions
  that are already type-checked. This implies "no free variables"
  + Values might be [neutral values](ast/enum.GenericNeutral.html): these values represents
    variable bindings that are not free but not reducible, like a parameter, or an expression
    that cannot be reduced due to the presence of this parameter
  + Values might be [closures](ast/enum.Closure.html): expression + context + parameter bindings
+ [Normal form syntax tree](check/read_back/enum.NormalExpression.html), aka normal forms.
  This is the output of the `read-back` functions
  + Details are introduced later. Personally, I consider this not necessary and ugly.

## Type-Checking

Mini-TT supports inferring types of simple expressions like applications, variable references, etc.
But not the case for even a bit more complicated structures, like lambdas.

### `checkD`

Each program is a sequence of definitions, each definition comes with a type signature and a body
expression. We check the definitions one by one, after checking each definition we add it to the
context and check the rest. For recursive definitions, we generate a neutral value before actually
checking it.

This part is trivial in Mini-TT, but minitt extended definitions with *prefix parameters*, which
are parameters present before the type signature and the body expression, resulting in a much more
complicated implementation.

+ [Outer wrapper](check/decl/fn.check_declaration.html)
+ [Recursive checking](check/decl/fn.check_recursive_declaration.html)
+ [Non-recursive checking](check/decl/fn.check_simple_declaration.html)
+ [Prefix parameters checking](check/decl/fn.check_lift_parameters.html)

### `check`

This is the so-called `instance of` check, the function name in Mini-TT paper is `check`.

All definitions in Mini-TT comes along with a type signature, Mini-TT tries to type-check the
signature and then try to match the body expression with the signature, using some hard-coded
patterns (relevant codes are in [check/expr.rs](check/expr/fn.check.html)), like if the type
is a pi-type and the value is a lambda, then we go on checking their bodies and types with the
parameter instantiated as a generated value then recursively check if the instantiated body
expression is an instance of the pi-type's return type; if the type is a sum type and the value
is a constructor call, then check if the constructor is present in the sum.

If all these hard-coded rules are not applicable, infer the expression type and perform a
[subtyping check](check/subtype/fn.check_subtype.html). This rule is an extension.
The subtyping check is basically doing some hard-coded comparisons as well.

If it still fails, [read back](check/read_back/) to
normal form and do a syntactic comparison with the `read-back`ed expected type signature.

### `checkI`

Try to infer the type of a given expression.

Cannot infer types of lambdas or other complicated expressions like nested function calls
(this situation has been improved a lot if you're glad to use prefix parameters).

### `checkT`

Check if an expression is a type expression.

Use some hard-coded rules and fallback to `check(expr, Type)`.

## Possible Extensions

Several extensions can be made apart from the improvements that have nothing to do with the
core type theory.
I'm listing all the possible extension, disregarding how hard can the implementation be.

+ Indexed inductive families
  + Dependent (co)pattern matching
  + Overlapping pattern matching
  + Props
  + Without-K
+ Quantitative Type Theory
  + Linear Type System
    + [Symmetric-Interaction-Calculus](https://github.com/MaiaVictor/Symmetric-Interaction-Calculus)
  + Affine Type System
+ First-class cases and sums
  + Record polymorphism
+ Cubical Type Theory
  + Already implemented in another Mini-TT dialect:
    [cubicaltt](https://github.com/mortberg/cubicaltt)
  + Cartesian Cubical Type Theory
  + De Morgan Cubical Type Theory
+ Coinduction and Guarded Recursion
+ Sized types (implicit?)
*/

/// Syntax: term, expression, context.
///
/// Methods are defined in `reduce`/`read_back` modules but their documents are here.
///
/// No dependency.
pub mod ast;
/// Reduction: eval and eval's friends.
///
/// Functions in this module are put into `impl` blocks, their docs can be found in:
///
/// + [Methods of `Pattern`](../syntax/enum.Pattern.html#methods)
/// + [Methods of `Value`](../syntax/enum.Value.html#methods)
/// + [Methods of `Telescope`](../syntax/enum.Telescope.html#methods)
/// + [Methods of `Closure`](../syntax/enum.Closure.html#methods)
/// + [Methods of `Expression`](../syntax/enum.Expression.html#methods)
///
/// Depends on module `syntax`.
pub mod eval;

/// Type checking: everything related to type-checking, including:<br/>
/// + Normal form and read-back functions
/// + The four type checking functions -- `checkI`, `checkD`, `check` and `checkT`.
/// + (extended) (sub)typing rules
///
/// Depends on module `syntax`.
pub mod check;

/// Pretty print utilities
#[cfg(feature = "pretty")]
pub mod pretty;

/// Parser, from text to AST and a bunch of utilities
#[cfg(feature = "parser")]
pub mod parser;
