use std::collections::BTreeMap;

use crate::read_back::*;
use crate::syntax::*;
use std::borrow::Cow;

pub type GammaRaw = BTreeMap<String, Value>;

/// `Gamma` in Mini-TT.<br/>
/// By doing this we get `lookupG` in Mini-TT for free.
pub type Gamma<'a> = Cow<'a, GammaRaw>;

/// `G` in Mini-TT.<br/>
/// Type-Checking Monad.
pub type TCM<T> = Result<T, String>;

/// `upG` in Mini-TT.<br/>
/// `Gamma |- p : t = u => Gamma’`<br/><br/>
/// `Cow` is used to simulate immutability.
pub fn update_gamma<'a>(
    gamma: Gamma<'a>,
    pattern: &Pattern,
    type_val: Value,
    val: Value,
) -> TCM<Gamma<'a>> {
    match pattern {
        Pattern::Pair(pattern_first, pattern_second) => match type_val {
            Value::Sigma(first, second) => {
                let (val_first, val_second) = val.destruct();
                let gamma = update_gamma(gamma, pattern_first, *first, val_first.clone())?;
                let second = second.instantiate(val_first);
                update_gamma(gamma, pattern_second, second, val_second)
            }
            _ => Err(format!("Cannot update Gamma by: {}", pattern)),
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
/// Type inference rule. More inferences are added here (maybe it's useful?).
pub fn check_infer(
    index: u32,
    context: Telescope,
    gamma: Gamma,
    expression: Expression,
) -> TCM<Value> {
    use crate::syntax::Expression::*;
    match expression {
        Unit => Ok(Value::One),
        Type | Void | One => Ok(Value::Type),
        Var(name) => gamma
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Unresolved reference `{}`", name).to_string()),
        First(pair) => match check_infer(index, context, gamma, *pair)? {
            Value::Sigma(first, _) => Ok(*first),
            e => Err(format!("Expected Sigma, got: {}", e).to_string()),
        },
        Second(pair) => match check_infer(index, context.clone(), gamma, *pair.clone())? {
            Value::Sigma(_, second) => Ok(second.instantiate(pair.eval(context).first())),
            e => Err(format!("Expected Sigma, got: {}", e).to_string()),
        },
        Application(function, argument) => {
            match check_infer(index, context.clone(), Cow::Borrowed(&gamma), *function)? {
                Value::Pi(input, output) => {
                    check(index, context.clone(), gamma, *argument.clone(), *input)?;
                    Ok(output.instantiate(argument.eval(context)))
                }
                e => Err(format!("Expected Pi, got: {}", e).to_string()),
            }
        }
        e => Err(format!("Cannot infer type of: {}", e)),
    }
}

/// `checkD` in Mini-TT.<br/>
/// Check if a declaration is well-typed and update the context.
pub fn check_declaration(
    index: u32,
    context: Telescope,
    gamma: Gamma,
    declaration: Declaration,
) -> TCM<Gamma> {
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
                up_var_rc(context.clone(), pattern.clone(), generated),
                fake_gamma,
                body.clone(),
                signature.clone(),
            )?;
            let declaration = Recursive(pattern.clone(), signature_plain, body.clone());
            let body = body.eval(up_dec_rc(context, declaration));
            update_gamma(gamma, &pattern, signature, body)
        }
    }
}

/// `checkT` in Mini-TT.<br/>
/// Check if an expression is a well-typed type expression.
pub fn check_type(index: u32, context: Telescope, gamma: Gamma, expression: Expression) -> TCM<()> {
    use crate::syntax::Expression::*;
    match expression {
        Sum(constructors) => check_sum_type(index, context, gamma, constructors),
        Pi(pattern, first, second) | Sigma(pattern, first, second) => {
            check_telescoped(index, context, gamma, pattern, *first, *second)
        }
        Type | Void | One => Ok(()),
        expression => check(index, context, gamma, expression, Value::Type),
    }
}

/// `check` in Mini-TT.<br/>
pub fn check(
    index: u32,
    context: Telescope,
    gamma: Gamma,
    expression: Expression,
    value: Value,
) -> TCM<()> {
    use crate::syntax::Expression as E;
    use crate::syntax::Value as V;
    match (expression, value) {
        (E::Unit, V::One) | (E::Type, V::Type) | (E::One, V::Type) => Ok(()),
        // There's nothing left to check.
        (E::Void, _) => Ok(()),
        (E::Lambda(pattern, body), V::Pi(signature, closure)) => {
            let generated = generate_value(index);
            let gamma = update_gamma(gamma, &pattern, *signature, generated.clone())?;
            check(
                index + 1,
                up_var_rc(context, pattern, generated.clone()),
                gamma,
                *body,
                closure.instantiate(generated),
            )
        }
        (E::Pair(first, second), V::Sigma(first_type, second_type)) => {
            check(
                index,
                context.clone(),
                Cow::Borrowed(&gamma),
                *first.clone(),
                *first_type,
            )?;
            check(
                index,
                context.clone(),
                gamma,
                *second,
                second_type.instantiate(first.eval(context)),
            )
        }
        (E::Constructor(name, body), V::Sum((constructors, telescope))) => {
            let constructor = *constructors
                .get(&name)
                .ok_or_else(|| format!("Invalid constructor: `{}`", name).to_string())?
                .clone();
            check(index, context, gamma, *body, constructor.eval(*telescope))
        }
        (E::Sum(constructors), V::Type) => check_sum_type(index, context, gamma, constructors),
        (E::Sigma(pattern, first, second), V::Type) | (E::Pi(pattern, first, second), V::Type) => {
            check_telescoped(index, context, gamma, pattern, *first, *second)
        }
        (E::Declaration(declaration, rest), rest_type) => {
            let gamma = check_declaration(index, context.clone(), gamma, *declaration)?;
            check(index, context, gamma, *rest, rest_type)
        }
        // I really wish to have box pattern here :(
        (E::Function(mut branches), V::Pi(sum, closure)) => match *sum {
            V::Sum((sum_branches, telescope)) => {
                for (name, clause) in sum_branches.into_iter() {
                    let constructor = *branches
                        .remove(&name)
                        .ok_or_else(|| format!("Missing clause for `{}`", name).to_string())?;
                    check(
                        index,
                        context.clone(),
                        Cow::Borrowed(&gamma),
                        *clause,
                        V::Pi(
                            Box::new(constructor.eval(*telescope.clone())),
                            Closure::Choice(Box::new(closure.clone()), name.clone()),
                        ),
                    )?;
                }
                if branches.is_empty() {
                    Ok(())
                } else {
                    let clauses: Vec<_> = branches.keys().map(|br| br.as_str()).collect();
                    Err(format!("Unexpected clauses: {}", clauses.join(", ")).to_string())
                }
            }
            not_sum_so_fall_through => check_infer(index, context, gamma, E::Function(branches))?
                .eq_normal(index, V::Pi(Box::new(not_sum_so_fall_through), closure)),
        },
        (expression, value) => {
            check_infer(index, context, gamma, expression)?.eq_normal(index, value)
        }
    }
}

/// To reuse code that checks if a sum type is well-typed between `check_type` and `check`
fn check_sum_type(index: u32, context: Telescope, gamma: Gamma, constructors: Branch) -> TCM<()> {
    for constructor in constructors.values().cloned() {
        check_type(index, context.clone(), Cow::Borrowed(&gamma), *constructor)?;
    }
    Ok(())
}

/// `checkMain` in Mini-TT.
pub fn check_main(expression: Expression) -> TCM<()> {
    check(0, nil_rc(), Default::default(), expression, Value::One)
}

/// To reuse code that checks if a sigma or a pi type is well-typed between `check_type` and `check`
fn check_telescoped(
    index: u32,
    context: Telescope,
    gamma: Gamma,
    pattern: Pattern,
    first: Expression,
    second: Expression,
) -> TCM<()> {
    check_type(index, context.clone(), Cow::Borrowed(&gamma), first.clone())?;
    let generated = generate_value(index);
    let gamma = update_gamma(
        Cow::Borrowed(&gamma),
        &pattern,
        first.eval(context.clone()),
        generated.clone(),
    )?;
    check_type(
        index + 1,
        up_var_rc(context, pattern, generated),
        gamma,
        second,
    )
}

#[cfg(test)]
mod tests {
    use crate::syntax::Declaration;
    use crate::syntax::Expression;
    use crate::syntax::Pattern;
    use crate::type_check::check_main;

    #[test]
    fn simple_check() {
        check_main(Expression::Declaration(
            Box::new(Declaration::Simple(
                Pattern::Unit,
                Expression::Type,
                Expression::One,
            )),
            Box::new(Expression::Void),
        ))
        .unwrap();
        check_main(Expression::Declaration(
            Box::new(Declaration::Simple(
                Pattern::Unit,
                Expression::Type,
                Expression::Unit,
            )),
            Box::new(Expression::Void),
        ))
        .unwrap_err();
    }
}
