/// Read back: read back functions and normal form definitions.
///
/// Converting terms to normal forms with de-bruijn indices so
/// we do not need to deal with alpha conversions when checking syntactic-equality.
///
/// Functions in this module are put into `impl for` blocks, their docs can be found in:
///
/// + [`ReadBack` of `Value`](../../syntax/enum.Value.html#impl-ReadBack)
/// + [`ReadBack` of `Telescope`](../../syntax/enum.GenericTelescope.html#impl-ReadBack)
/// + [`ReadBack` of `Closure`](../../syntax/enum.Closure.html#impl-ReadBack)
///
/// Depends on modules `syntax`.
pub mod read_back;

/// Type-Checking Monad: context, state and error.
///
/// Typing context (`Gamma`) and its updater, the type-checking error and its pretty-printer
///
/// Depends on module `syntax`.
#[macro_use]
pub mod tcm;

/// Subtyping check: fallback rules of "instance of" checks: infer the expression's type and check
/// if it's the subtype of the expected type.
///
/// Depends on modules `syntax` and `read_back`.
pub mod subtype;

/// Expression checker: infer, instance-of check, normal-form comparison, subtyping, etc.
///
/// Depends on modules `syntax`, `read_back` and `tcm`.
pub mod expr;

/// Declaration checker: for prefix parameters, simple declarations and recursive declarations.
///
/// Depends on modules `syntax`, `expr` and `tcm`.
/// $$
/// \textnormal{checkD}\quad \rho,\Gamma\vdash_l D\Rightarrow \Gamma'
/// $$
pub mod decl;

use self::decl::check_declaration;
use self::expr::{check, check_infer};
use self::tcm::{TCM, TCS};
use crate::ast::{Declaration, Expression, Value};

/// `checkMain` in Mini-TT.
pub fn check_main<'a>(expression: Expression) -> TCM<TCS<'a>> {
    check_contextual(Default::default(), expression)
}

/// For REPL: check an expression under an existing context
pub fn check_contextual(tcs: TCS, expression: Expression) -> TCM<TCS> {
    check(0, tcs, expression, Value::One)
}

/// For REPL: infer the type of an expression under an existing context
pub fn check_infer_contextual(tcs: TCS, expression: Expression) -> TCM<Value> {
    check_infer(0, tcs, expression)
}

/// Similar to `checkMain` in Mini-TT, but for a declaration.
pub fn check_declaration_main<'a>(declaration: Declaration) -> TCM<TCS<'a>> {
    check_declaration(0, Default::default(), declaration)
}

#[cfg(test)]
mod tests;
