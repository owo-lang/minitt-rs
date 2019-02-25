/// Reduction: eval and eval's friends.<br/>
/// Functions are basically put into `impl` blocks, their docs are not inside this module.
pub mod reduce;
/// Syntax: term, expression, context.
/// Methods are inside the `reduce` module.
pub mod syntax;

/// Normal form: when we read back, we get a normal form expression.
pub mod normal;

/// Type check: all about type checking.
pub mod check;
