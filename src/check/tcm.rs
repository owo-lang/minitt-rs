use std::fmt::{Display, Formatter, Error};
use crate::syntax::{nil_rc, Telescope, Pattern, Value};
use std::collections::BTreeMap;
use std::borrow::Cow;

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

/// `upG` in Mini-TT.<br/>
/// `Gamma |- p : t = u => Gamma’`<br/><br/>
/// `Cow` is used to simulate immutability.
pub fn update_gamma<'a>(
    gamma: Gamma<'a>,
    pattern: &Pattern,
    type_val: Value,
    generated_val: Value,
) -> TCM<Gamma<'a>> {
    match pattern {
        Pattern::Pair(pattern_first, pattern_second) => match type_val {
            Value::Sigma(first, second) => {
                let (val_first, val_second) = generated_val.destruct();
                let gamma = update_gamma(gamma, pattern_first, *first, val_first.clone())?;
                let second = second.instantiate(val_first);
                update_gamma(gamma, pattern_second, second, val_second)
            }
            _ => TCE::default_error(format!("Cannot update Gamma by: `{}`.", pattern)),
        },
        Pattern::Var(name) => {
            let mut gamma = gamma.into_owned();
            gamma.insert(name.clone(), type_val);
            Ok(Cow::Owned(gamma))
        }
        Pattern::Unit => Ok(gamma),
    }
}
