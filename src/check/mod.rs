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

/// Type-Checking Monad: context, state and error.
///
/// Typing context (`Gamma`) and its updater, the type-checking error and its pretty-printer
///
/// Depends on module `syntax`.
pub mod tcm;

/// Declaration checker: for prefix parameters, simple declarations and recursive declarations.
///
/// Depends on modules `syntax` and `tcm`.
pub mod decl;
