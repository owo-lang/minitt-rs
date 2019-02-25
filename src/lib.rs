/// Reduction: eval and eval's friends.<br/>
/// Functions are basically put into `impl` blocks, their docs are not inside this module.
pub mod reduce;
/// Syntax: term, expression, context.
/// Methods are defined in `reduce`/`read_back` modules but their documents are here.
pub mod syntax;

/// Normal form: when we read back, we get a normal form expression.<br/>
/// Functions are basically put into `impl` blocks, their docs are not inside this module.
pub mod normal;
/// Read back: read back functions, converting terms to normal forms with de-bruijn indices so
/// we do not need to deal with alpha conversions.
pub mod read_back;

/// Type checking: the four type checking functions -- `checkI`, `checkD`, `check` and `checkT`.
pub mod type_check;
