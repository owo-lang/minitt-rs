use crate::ast::{nil_rc, Closure, Pattern, Telescope, Value};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::{Display, Error, Formatter};

/// Type-Checking context. Name as key, type of the declaration as value.
pub type GammaRaw = BTreeMap<String, Value>;

/// `Gamma` in Mini-TT.<br/>
/// By doing this we get `lookupG` in Mini-TT for free.
pub type Gamma<'a> = Cow<'a, GammaRaw>;

/// Type-Checking Error.
#[derive(Clone, Debug)]
pub enum TCE {
    Textual(String),
    Located(String, Pattern),
}

/// `G` in Mini-TT.<br/>
/// Type-Checking Monad.
pub type TCM<T> = Result<T, TCE>;

/// Type-Checking State~~, not "Theoretical Computer Science"~~.<br/>
/// This is not present in Mini-TT.
pub type TCS<'a> = (Gamma<'a>, Telescope);

/// Empty `TCS`.
pub fn default_state<'a>() -> TCS<'a> {
    (Default::default(), nil_rc())
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
            TCE::Textual(s) => {
                f.write_str(s.as_str())?;
                f.write_str("\n")?;
                f.write_str("At unknown location.")
            }
            TCE::Located(s, pattern) => {
                f.write_str(s.as_str())?;
                f.write_str("\n")?;
                f.write_str("When checking the declaration of `")?;
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
                _ => TCE::default_error(format!("Cannot update Gamma by: `{}`.", $pattern)),
            },
            Pattern::Var(name) => update_gamma_by_var($gamma, $type_val, name),
            Pattern::Unit => Ok($gamma),
        }
    };
}

/// Move version of `upG` in Mini-TT.<br/>
/// `Gamma |- p : t = u => Gamma’`<br/><br/>
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

fn update_gamma_by_var<'a>(gamma: Gamma<'a>, type_val: Value, name: &String) -> TCM<Gamma<'a>> {
    let mut gamma = gamma.into_owned();
    gamma.insert(name.clone(), type_val);
    Ok(Cow::Owned(gamma))
}
