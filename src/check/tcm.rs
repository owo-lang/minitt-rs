use core::fmt::Write;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Display, Error, Formatter};

use either::{Either, Left, Right};

use super::read_back::NormalExpression;
use crate::ast::{nil_rc, up_var_rc, Closure, Expression, Level, Pattern, Telescope, Value};

/// Since we have no place to document `lookupG` I'll put it here:
/// $$
/// \frac{}{(\Gamma, a:t)(x)\rightarrow t}
/// \quad
/// \frac{\Gamma(x) \rightarrow t}
/// {(\Gamma, y:t')(x)\rightarrow t} y \neq x
/// $$
/// Type-Checking context. Name as key, type of the declaration as value.
pub type GammaRaw = BTreeMap<String, Value>;

/// $\Gamma ::= () \ | \ \Gamma, x : t$,
/// `Gamma` in Mini-TT.<br/>
/// By aliasing `BTreeMap` to `Gamma`, we get `lookupG` in Mini-TT for free.
pub type Gamma<'a> = Cow<'a, GammaRaw>;

/// Type-Checking Error.
#[derive(Clone, Debug)]
pub enum TCE {
    Textual(String),
    UpdateGammaFailed(Pattern),
    CannotInfer(Expression),
    UnresolvedName(String),
    InvalidConstructor(String),
    MissingCase(String),
    UnexpectedCases(String),
    /// Reaching somewhere that is not expected to reach.
    Unreachable(&'static str, u32, u32),
    WantSumBut(Either<Value, Expression>),
    DuplicateBranch(String),
    WantSigmaBut(Value),
    /// We can get the argument of application here, to better report error.
    WantPiBut(Value, Expression),
    /// Actually first value, expected second value.
    TypeMismatch(Value, Value),
    /// Want a type's type, but unfortunately it's not.
    NotTypeType(Value),
    /// Actually first level, expected second level.
    LevelMismatch(Level, Level),
    /// First argument is inferred value, second is expected.
    ReadBackTypeMismatch(NormalExpression, NormalExpression),
    Located(Box<TCE>, Pattern),
}

/// `G` in Mini-TT.<br/>
/// Type-Checking Monad.
pub type TCM<T> = Result<T, TCE>;

/// Type-Checking State <del>, not "Theoretical Computer Science"</del>.<br/>
/// This is not present in Mini-TT.
#[derive(Debug)]
pub struct TCS<'a> {
    pub gamma: Gamma<'a>,
    pub context: Telescope,
}

impl<'a> TCS<'a> {
    pub fn new(gamma: Gamma<'a>, context: Telescope) -> Self {
        Self { gamma, context }
    }

    /// Since `context` is ref-counted, it's gonna be cheap to clone.
    pub fn context(&self) -> Telescope {
        self.context.clone()
    }

    pub fn update(self, pattern: Pattern, type_val: Value, body: Value) -> TCM<TCS<'a>> {
        Ok(TCS {
            gamma: update_gamma_borrow(self.gamma, &pattern, type_val.clone(), &body)?,
            context: up_var_rc(self.context, pattern.clone(), body),
        })
    }
}

impl<'a> Default for TCS<'a> {
    fn default() -> Self {
        Self::new(Default::default(), nil_rc())
    }
}

/// Cannot be an implementation of `Clone` due to lifetime requirement.
#[macro_export]
macro_rules! tcs_borrow {
    ($tcs:expr) => {{
        let TCS { gamma, context } = &$tcs;
        TCS::new(std::borrow::Cow::Borrowed(&*gamma), context.clone())
    }};
}

/// Records the source code location that the error occurs, just like `Agda.Util.Impossible`
#[macro_export]
macro_rules! tce_unreachable {
    () => {
        TCE::Unreachable(file!(), line!(), column!())
    };
}

impl TCE {
    /// Default `TCE`
    pub fn default_error<T>(str: String) -> TCM<T> {
        Err(TCE::Textual(str))
    }
}

impl Display for TCE {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            TCE::Textual(s) => f.write_str(s.as_str()),
            TCE::UpdateGammaFailed(pattern) => {
                f.write_str("Cannot update Gamma by: `")?;
                pattern.fmt(f)?;
                f.write_str("`.")
            }
            TCE::CannotInfer(expression) => {
                f.write_str("Cannot infer type of: `")?;
                expression.fmt(f)?;
                f.write_str("`.")
            }
            TCE::UnresolvedName(name) => {
                f.write_str("Unresolved reference: `")?;
                f.write_str(name.as_str())?;
                f.write_str("`.")
            }
            TCE::InvalidConstructor(name) => {
                f.write_str("Invalid constructor: `")?;
                f.write_str(name.as_str())?;
                f.write_str("`.")
            }
            TCE::WantSigmaBut(expression) => {
                f.write_str("Expected \u{03A3} type, instead got: `")?;
                expression.fmt(f)?;
                f.write_str("`.")
            }
            TCE::WantSumBut(either) => {
                f.write_str("Expected Sum type, instead got: `")?;
                match either {
                    Left(value) => value.fmt(f)?,
                    Right(expression) => expression.fmt(f)?,
                };
                f.write_str("`.")
            }
            TCE::DuplicateBranch(branch) => {
                f.write_str("Found duplicated branch: `")?;
                f.write_str(branch)?;
                f.write_str("`.")
            }
            TCE::Unreachable(file, line, column) => {
                f.write_str(
                    "An internal error has occurred. Please report this as a bug.\n\
                     Location of the error: ",
                )?;
                file.fmt(f)?;
                f.write_str(", line ")?;
                line.fmt(f)?;
                f.write_str(", column ")?;
                column.fmt(f)?;
                f.write_char('.')
            }
            TCE::WantPiBut(expression, argument) => {
                f.write_str("Expected \u{03A0} type, instead got: `")?;
                expression.fmt(f)?;
                f.write_str("`\nWhen checking the application whose argument is `")?;
                argument.fmt(f)?;
                f.write_str("`.")
            }
            TCE::NotTypeType(value) => {
                f.write_str("Expected a type expression, instead got: `")?;
                value.fmt(f)?;
                f.write_str("`.")
            }
            TCE::LevelMismatch(actual, expected) => {
                f.write_str("Expected a type expression at level `")?;
                expected.fmt(f)?;
                f.write_str("`, instead got one at level: `")?;
                actual.fmt(f)?;
                f.write_str("`.")
            }
            TCE::MissingCase(name) => {
                f.write_str("Missing case-split: `")?;
                f.write_str(name.as_str())?;
                f.write_str("`.")
            }
            TCE::UnexpectedCases(joined_name) => {
                f.write_str("Unexpected case-split: `")?;
                f.write_str(joined_name.as_str())?;
                f.write_str("`.")
            }
            TCE::ReadBackTypeMismatch(inferred, expected) => mismatch(f, inferred, expected),
            TCE::TypeMismatch(inferred, expected) => mismatch(f, inferred, expected),
            TCE::Located(wrapped, pattern) => {
                wrapped.fmt(f)?;
                f.write_str("\nWhen checking the declaration of `")?;
                pattern.fmt(f)?;
                f.write_str("`.")
            }
        }
    }
}

macro_rules! update_gamma {
    ($gamma:expr, $pattern:expr, $type_val:expr, $clone:expr) => {
        match $pattern {
            Pattern::Pair(pattern_first, pattern_second) => match $type_val {
                Value::Sigma(first, second) => update_gamma_by_pair(
                    $gamma,
                    pattern_first,
                    pattern_second,
                    *first,
                    second,
                    $clone,
                ),
                _ => Err(TCE::UpdateGammaFailed($pattern.clone())),
            },
            Pattern::Var(name) => update_gamma_by_var($gamma, $type_val, name),
            Pattern::Unit => Ok($gamma),
        }
    };
}

/// Move version of `upG` in Mini-TT.<br/>
/// $$
/// \frac{}{\Gamma \vdash x:t=v\Rightarrow \Gamma,x:t}
/// \quad
/// \frac{}{\Gamma \vdash \_:t=v\Rightarrow \Gamma}
/// $$
/// $$
/// \frac{\Gamma \vdash p_1:t_1=v.1 \Rightarrow \Gamma_1
///   \quad \Gamma \vdash p_2:\textsf{inst}\ g(v.1)=v.2 \Rightarrow \Gamma_2}
///  {\Gamma \vdash (p_1,p_2):\Sigma\ t_1\ g=v\Rightarrow \Gamma_2}
/// $$
/// `Cow` is used to simulate immutability.
pub fn update_gamma<'a>(
    gamma: Gamma<'a>,
    pattern: &Pattern,
    type_val: Value,
    body: Value,
) -> TCM<Gamma<'a>> {
    update_gamma!(gamma, pattern, type_val, body)
}

/// Borrow version of `upG` in Mini-TT.
pub fn update_gamma_borrow<'a>(
    gamma: Gamma<'a>,
    pattern: &Pattern,
    type_val: Value,
    body: &Value,
) -> TCM<Gamma<'a>> {
    update_gamma!(gamma, pattern, type_val, body.clone())
}

/// Lazy version of `upG` in Mini-TT.
pub fn update_gamma_lazy<'a>(
    gamma: Gamma<'a>,
    pattern: &Pattern,
    type_val: Value,
    body: impl FnOnce() -> Value,
) -> TCM<Gamma<'a>> {
    update_gamma!(gamma, pattern, type_val, body())
}

/// Some minor helper specialized from other functions.
fn update_gamma_by_pair<'a>(
    gamma: Gamma<'a>,
    pattern_first: &Box<Pattern>,
    pattern_second: &Box<Pattern>,
    first: Value,
    second: Closure,
    generated_val: Value,
) -> TCM<Gamma<'a>> {
    let (val_first, val_second) = generated_val.destruct();
    let gamma = update_gamma(gamma, pattern_first, first, val_first.clone())?;
    let second = second.instantiate(val_first);
    update_gamma(gamma, pattern_second, second, val_second)
}

/// Some minor helper specialized from other functions.
fn update_gamma_by_var<'a>(gamma: Gamma<'a>, type_val: Value, name: &String) -> TCM<Gamma<'a>> {
    let mut gamma = gamma.into_owned();
    gamma.insert(name.clone(), type_val);
    Ok(Cow::Owned(gamma))
}

#[inline]
fn mismatch<E: Display>(f: &mut Formatter, inferred: &E, expected: &E) -> Result<(), Error> {
    f.write_str("Type mismatch: expected `")?;
    expected.fmt(f)?;
    f.write_str("`, got (inferred): `")?;
    inferred.fmt(f)?;
    f.write_str("`.")
}
