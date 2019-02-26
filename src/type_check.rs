use std::collections::BTreeMap;

use crate::read_back::generate_value;
use crate::reduce::*;
use crate::syntax::*;
use std::borrow::Cow;
use std::rc::Rc;

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
                let (val_first, val_second) = val.destruct();
                let gamma = update_gamma(gamma, pattern_first, *first, val_first.clone())?;
                let second = second.instantiate(val_first);
                update_gamma(gamma, pattern_second, second, val_second)
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
pub fn check_declaration<Name: DebuggableNameTrait>(
    index: u32,
    context: Telescope<Name>,
    gamma: Gamma<Name>,
    declaration: Declaration<Name>,
) -> TCM<Gamma<Name>> {
    match declaration {
        Declaration::Simple(pattern, signature, body) => {
            check_type(
                index,
                context.clone(),
                Cow::Borrowed(&gamma),
                signature.clone(),
            )?;
            let signature = signature.eval(context.clone());
            check(
                index,
                context.clone(),
                Cow::Borrowed(&gamma),
                body.clone(),
                signature.clone(),
            )?;
            update_gamma(gamma, &pattern, signature, body.eval(context))
        }
        Declaration::Recursive(pattern, signature, body) => {
            check_type(
                index,
                context.clone(),
                Cow::Borrowed(&gamma),
                signature.clone(),
            )?;
            let signature_plain = signature.clone();
            let signature = signature.eval(context.clone());
            let generated = generate_value(index);
            let fake_gamma = update_gamma(
                Cow::Borrowed(&gamma),
                &pattern,
                signature.clone(),
                generated.clone(),
            )?;
            check(
                index + 1,
                GenericTelescope::up_var_rc(context.clone(), pattern.clone(), generated),
                Cow::Borrowed(&fake_gamma),
                body.clone(),
                signature.clone(),
            )?;
            let declaration =
                Declaration::Recursive(pattern.clone(), signature_plain, body.clone());
            update_gamma(
                gamma,
                &pattern,
                signature,
                body.eval(GenericTelescope::up_dec_rc(context, declaration)),
            )
        }
    }
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
            check_type(
                index,
                context.clone(),
                Cow::Borrowed(&gamma),
                *first.clone(),
            )?;
            let gamma = update_gamma(
                Cow::Borrowed(&gamma),
                &pattern,
                first.eval(context.clone()),
                generate_value(index),
            )?;
            check_type(
                index + 1,
                Rc::new(GenericTelescope::UpVar(
                    context,
                    pattern,
                    generate_value(index),
                )),
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
