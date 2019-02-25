use std::collections::BTreeMap;

use crate::reduce::*;
use crate::syntax::*;

/// `Gamma` in Mini-TT.
/// By doing this we get `lookupG` in Mini-TT for free.
type Gamma<Name> = BTreeMap<Name, Value<Name>>;

/// `genV` in Mini-TT.
pub fn generate_value<Name: NameTrait>(id: u32) -> Value<Name> {
    Value::Neutral(Neutral::Generated(id))
}

/// `upG` in Mini-TT.
/// Gamma |- p : t = u => Gamma’
///
/// However, since Rust is an imperative language, we use mutable reference instead of making it
/// monadic.
pub fn update_gamma<'a, Name: DebuggableNameTrait>(
    gamma: &'a mut Gamma<Name>,
    pattern: &Pattern<Name>,
    type_val: Value<Name>,
    val: Value<Name>,
) -> Result<&'a mut Gamma<Name>, String> {
    match pattern {
        Pattern::Pair(pattern_first, pattern_second) => match type_val {
            Value::Sigma(first, second) => {
                let val_first = val.clone().first();
                update_gamma(gamma, pattern_first, *first, val_first.clone())?;
                // Gamma is updated here -- since it's passed as a mutable reference.
                update_gamma(
                    gamma,
                    pattern_second,
                    second.instantiate(val_first),
                    val.second(),
                )
            }
            _ => Err(format!("Cannot update Gamma by: {:?}", pattern)),
        },
        Pattern::Var(name) => {
            gamma.insert(name.clone(), type_val);
            Ok(gamma)
        }
        Pattern::Unit => Ok(gamma),
    }
}
