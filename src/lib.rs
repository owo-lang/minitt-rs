#[cfg(feature = "parser")]
extern crate pest_derive;

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
pub mod reduce;
/// Syntax: term, expression, context.
///
/// Methods are defined in `reduce`/`read_back` modules but their documents are here.
///
/// No dependency.
pub mod syntax;

/// Normal form: when we read back, we get a normal form expression.
///
/// Depends on module `syntax`.
pub mod normal;
/// Read back: read back functions.
///
/// Converting terms to normal forms with de-bruijn indices so
/// we do not need to deal with alpha conversions.
///
/// Functions in this module are put into `impl for` blocks, their docs can be found in:
///
/// + [`ReadBack` of `Value`](../syntax/enum.Value.html#impl-ReadBack)
/// + [`ReadBack` of `Telescope`](../syntax/enum.Telescope.html#impl-ReadBack)
/// + [`ReadBack` of `Closure`](../syntax/enum.Closure.html#impl-ReadBack)
///
/// Depends on modules `syntax` and `normal`.
pub mod read_back;

/// Type checking: the four type checking functions -- `checkI`, `checkD`, `check` and `checkT`.
///
/// Depends on modules `syntax`, `normal`, `reduce` and `read_back`.
pub mod type_check;

/// Pretty print utilities
#[cfg(feature = "pretty")]
pub mod pretty;

/// Parser, from text to AST and a bunch of utilities
#[cfg(feature = "parser")]
pub mod parser;
