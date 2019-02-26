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
/// Type inference rule. More inferences are added here (maybe it's useful?)
pub fn check_infer<Name: DebuggableNameTrait>(
    index: u32,
    context: Telescope<Name>,
    gamma: Gamma<Name>,
    expression: Expression<Name>,
) -> TCM<Value<Name>> {
    match expression {
        Expression::Unit => Ok(Value::One),
        Expression::One => Ok(Value::Type),
        Expression::Type => Ok(Value::Type),
        Expression::Void => Ok(Value::Type),
        Expression::Var(name) => gamma
            .get(&name)
            .map(|value| value.clone())
            .ok_or(format!("Unresolved reference {:?}", name).to_string()),
        Expression::First(pair) => match check_infer(index, context, gamma, *pair)? {
            Value::Sigma(first, _) => Ok(*first),
            e => Err(format!("Expected Sigma, got: {:?}", e).to_string()),
        },
        Expression::Second(pair) => {
            match check_infer(index, context.clone(), gamma, *pair.clone())? {
                Value::Sigma(_, second) => Ok(second.instantiate(pair.eval(context).first())),
                e => Err(format!("Expected Sigma, got: {:?}", e).to_string()),
            }
        }
        Expression::Application(function, argument) => {
            match check_infer(index, context.clone(), Cow::Borrowed(&gamma), *function)? {
                Value::Pi(input, output) => {
                    check(index, context.clone(), gamma, *argument.clone(), *input)?;
                    Ok(output.instantiate(argument.eval(context)))
                }
                e => Err(format!("Expected Pi, got: {:?}", e).to_string()),
            }
        }
        e => Err(format!("Cannot infer type of: {:?}", e)),
    }
}

/// `checkD` in Mini-TT.<br/>
pub fn check_declaration<Name: DebuggableNameTrait>(
    index: u32,
    context: Telescope<Name>,
    gamma: Gamma<Name>,
    declaration: Declaration<Name>,
) -> TCM<Gamma<Name>> {
    use crate::syntax::Declaration::*;
    match declaration {
        Simple(pattern, signature, body) => {
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
        Recursive(pattern, signature, body) => {
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
                fake_gamma,
                body.clone(),
                signature.clone(),
            )?;
            let declaration = Recursive(pattern.clone(), signature_plain, body.clone());
            let body = body.eval(GenericTelescope::up_dec_rc(context, declaration));
            update_gamma(gamma, &pattern, signature, body)
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
                GenericTelescope::up_var_rc(context, pattern, generate_value(index)),
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
