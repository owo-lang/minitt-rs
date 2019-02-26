use std::collections::BTreeMap;

use crate::read_back::generate_value;
use crate::reduce::*;
use crate::syntax::*;
use std::borrow::Cow;

pub type GammaRaw<Name> = BTreeMap<Name, Value<Name>>;

/// `Gamma` in Mini-TT.<br/>
/// By doing this we get `lookupG` in Mini-TT for free.
pub type Gamma<'a, Name> = Cow<'a, GammaRaw<Name>>;

/// `G` in Mini-TT.<br/>
/// Type-Checking Monad.
pub type TCM<T> = Result<T, String>;

/// `upG` in Mini-TT.<br/>
/// `Gamma |- p : t = u => Gamma’`<br/><br/>
/// However, since Rust is an imperative language, we use mutable reference instead of making it
/// monadic.
pub fn update_gamma<'a, Name: DebuggableNameTrait>(
    gamma: Gamma<'a, Name>,
    pattern: &Pattern<Name>,
    type_val: Value<Name>,
    val: Value<Name>,
) -> TCM<Gamma<'a, Name>> {
    match pattern {
        Pattern::Pair(pattern_first, pattern_second) => match type_val {
            Value::Sigma(first, second) => {
                let val_first = val.clone().first();
                let gamma = update_gamma(gamma, pattern_first, *first, val_first.clone())?;
                let second = second.instantiate(val_first);
                update_gamma(gamma, pattern_second, second, val.second())
            }
            _ => Err(format!("Cannot update Gamma by: {:?}", pattern)),
        },
        Pattern::Var(name) => {
            let mut gamma = gamma.into_owned();
            gamma.insert(name.clone(), type_val);
            Ok(Cow::Owned(gamma))
        }
        Pattern::Unit => Ok(gamma),
    }
}

/// `checkI` in Mini-TT.<br/>
pub fn check_infer<'a, Name: DebuggableNameTrait>(
    index: u32,
    context: Telescope<Name>,
    gamma: Gamma<'a, Name>,
    expression: Expression<Name>,
) -> TCM<Value<Name>> {
    unimplemented!()
}

/// `checkD` in Mini-TT.<br/>
pub fn check_declaration<'a, Name: DebuggableNameTrait>(
    index: u32,
    context: Telescope<Name>,
    gamma: Gamma<'a, Name>,
    declaration: Declaration<Name>,
) -> TCM<Gamma<'a, Name>> {
    unimplemented!()
}

/// `checkT` in Mini-TT.<br/>
pub fn check_type<Name: DebuggableNameTrait>(
    index: u32,
    context: Telescope<Name>,
    gamma: Gamma<Name>,
    expression: Expression<Name>,
) -> TCM<()> {
    use crate::syntax::Expression::*;
    match expression {
        Pi(pattern, first, second) | Sigma(pattern, first, second) => {
            check_type(index, context, Cow::Borrowed(&gamma), *first.clone())?;
            let gamma = update_gamma(gamma, &pattern, first.eval(context), generate_value(index))?;
            check_type(
                index + 1,
                &mut GenericTelescope::UpVar(Box::new(*context.clone()), pattern, generate_value(index)),
                gamma,
                *second,
            )
        }
        Type => Ok(()),
        expression => check(index, context, gamma, expression, Value::Type),
    }
}

/// `check` in Mini-TT.<br/>
pub fn check<Name: DebuggableNameTrait>(
    index: u32,
    context: Telescope<Name>,
    gamma: Gamma<Name>,
    expression: Expression<Name>,
    value: Value<Name>,
) -> TCM<()> {
    unimplemented!()
}
