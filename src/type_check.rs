use std::collections::BTreeMap;

use crate::read_back::*;
use crate::syntax::*;
use std::borrow::Cow;

/// Type-Checking context. Name as key, type of the declaration as value.
pub type GammaRaw = BTreeMap<String, Value>;

/// `Gamma` in Mini-TT.<br/>
/// By doing this we get `lookupG` in Mini-TT for free.
pub type Gamma<'a> = Cow<'a, GammaRaw>;

/// `G` in Mini-TT.<br/>
/// Type-Checking Monad.
pub type TCM<'a, T> = Result<T, (TCS<'a>, String)>;

/// Type-Checking State~~, not "Theoretical Computer Science"~~.<br/>
/// This is not present in Mini-TT.
pub type TCS<'a> = (Gamma<'a>, Telescope);

/// Empty `TCS`.
pub fn default_state<'a>() -> TCS<'a> {
    (Default::default(), nil_rc())
}

/// `upG` in Mini-TT.<br/>
/// `Gamma |- p : t = u => Gamma’`<br/><br/>
/// `Cow` is used to simulate immutability.
pub fn update_gamma<'a>(
    gamma: Gamma<'a>,
    pattern: &Pattern,
    type_val: Value,
    val: Value,
) -> TCM<'a, Gamma<'a>> {
    match pattern {
        Pattern::Pair(pattern_first, pattern_second) => match type_val {
            Value::Sigma(first, second) => {
                let (val_first, val_second) = val.destruct();
                let gamma = update_gamma(gamma, pattern_first, *first, val_first.clone())?;
                let second = second.instantiate(val_first);
                update_gamma(gamma, pattern_second, second, val_second)
            }
            _ => Err((
                (gamma, nil_rc()),
                format!("Cannot update Gamma by: {}", pattern),
            )),
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
pub fn check_infer(index: u32, (gamma, context): TCS, expression: Expression) -> TCM<Value> {
    use crate::syntax::Expression::*;
    match expression {
        Unit => Ok(Value::One),
        Type | Void | One => Ok(Value::Type),
        Var(name) => match gamma
            .get(&name)
            .cloned() {
            Some(var) => Ok(var),
            None => Err(((gamma, context), format!("Unresolved reference `{}`", name))),
        }
        Pair(left, right) => {
            let left = check_infer(index, (Cow::Borrowed(&gamma), context.clone()), *left)?;
            let right = check_infer(index, (Cow::Borrowed(&gamma), context.clone()), *right)?;
            Ok(Value::Sigma(
                Box::new(left),
                Closure::Value(Box::new(right)),
            ))
        }
        First(pair) => match check_infer(index, (Cow::Borrowed(&gamma), context), *pair)? {
            Value::Sigma(first, _) => Ok(*first),
            e => Err(((gamma, context), format!("Expected Sigma, got: {}", e))),
        },
        Second(pair) => match check_infer(
            index,
            (Cow::Borrowed(&gamma), context.clone()),
            *pair.clone(),
        )? {
            Value::Sigma(_, second) => Ok(second.instantiate(pair.eval(context).first())),
            e => Err(((gamma, context), format!("Expected Sigma, got: {}", e))),
        },
        Application(function, argument) => {
            match check_infer(index, (Cow::Borrowed(&gamma), context.clone()), *function)? {
                Value::Pi(input, output) => {
                    check(index, (gamma, context.clone()), *argument.clone(), *input)?;
                    Ok(output.instantiate(argument.eval(context)))
                }
                e => Err(((gamma, context), format!("Expected Pi, got: {}", e))),
            }
        }
        e => Err(((gamma, context), format!("Cannot infer type of: {}", e))),
    }
}

/// `checkD` in Mini-TT.<br/>
/// Check if a declaration is well-typed and update the context.
pub fn check_declaration(
    index: u32,
    (gamma, context): TCS,
    declaration: Declaration,
) -> TCM<Gamma> {
    use crate::syntax::Declaration::*;
    match declaration {
        Simple(pattern, signature, body) => {
            check_type(
                index,
                (Cow::Borrowed(&gamma), context.clone()),
                signature.clone(),
            )?;
            let signature = signature.eval(context.clone());
            check(
                index,
                (Cow::Borrowed(&gamma), context.clone()),
                body.clone(),
                signature.clone(),
            )?;
            update_gamma(gamma, &pattern, signature, body.eval(context))
        }
        Recursive(pattern, signature, body) => {
            check_type(
                index,
                (Cow::Borrowed(&gamma), context.clone()),
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
            let fake_context = up_var_rc(context.clone(), pattern.clone(), generated);
            check(
                index + 1,
                (fake_gamma, fake_context),
                body.clone(),
                signature.clone(),
            )?;
            // Just a self-clone
            // FIXME: if we put a self **before type-checking** (which requires special treatment)
            // into the context, self-reference are not gonna get resolved.
            let declaration = Recursive(pattern.clone(), signature_plain, body.clone());
            let body = body.eval(up_dec_rc(context, declaration));
            update_gamma(gamma, &pattern, signature, body)
        }
    }
}

/// `checkT` in Mini-TT.<br/>
/// Check if an expression is a well-typed type expression.
pub fn check_type(index: u32, (gamma, context): TCS, expression: Expression) -> TCM<TCS> {
    use crate::syntax::Expression::*;
    match expression {
        Sum(constructors) => check_sum_type(index, (gamma, context), constructors),
        Pi(pattern, first, second) | Sigma(pattern, first, second) => {
            check_telescoped(index, (gamma, context), pattern, *first, *second)
        }
        Type | Void | One => Ok((gamma, context)),
        expression => check(index, (gamma, context), expression, Value::Type),
    }
}

/// `check` in Mini-TT.<br/>
/// However, telescope and gamma are preserved for REPL use.
pub fn check(index: u32, (gamma, context): TCS, expression: Expression, value: Value) -> TCM<TCS> {
    use crate::syntax::Expression as E;
    use crate::syntax::Value as V;
    match (expression, value) {
        (E::Unit, V::One) | (E::Type, V::Type) | (E::One, V::Type) => Ok((gamma, context)),
        // There's nothing left to check.
        (E::Void, _) => Ok((gamma, context)),
        (E::Lambda(pattern, body), V::Pi(signature, closure)) => {
            let generated = generate_value(index);
            let gamma = update_gamma(gamma, &pattern, *signature, generated.clone())?;
            check(
                index + 1,
                (gamma, up_var_rc(context, pattern, generated.clone())),
                *body,
                closure.instantiate(generated),
            )
        }
        (E::Pair(first, second), V::Sigma(first_type, second_type)) => {
            check(
                index,
                (Cow::Borrowed(&gamma), context.clone()),
                *first.clone(),
                *first_type,
            )?;
            check(
                index,
                (gamma, context.clone()),
                *second,
                second_type.instantiate(first.eval(context)),
            )
        }
        (E::Constructor(name, body), V::Sum((constructors, telescope))) => {
            let constructor = match constructors
                .get(&name) {
                Some(cons) => *cons.clone(),
                None => return Err(((gamma, context), format!("Invalid constructor: `{}`", name))),
            };
            // FIXME: this is a workaround for recursive types
            check(
                index,
                (gamma, context.clone()),
                *body,
                constructor.eval(context),
            )
        }
        (E::Sum(constructors), V::Type) => check_sum_type(index, (gamma, context), constructors),
        (E::Sigma(pattern, first, second), V::Type) | (E::Pi(pattern, first, second), V::Type) => {
            check_telescoped(index, (gamma, context), pattern, *first, *second)
        }
        (E::Declaration(declaration, rest), rest_type) => {
            let gamma = check_declaration(index, (gamma, context.clone()), *declaration.clone())?;
            check(
                index,
                (gamma, up_dec_rc(context, *declaration)),
                *rest,
                rest_type,
            )
        }
        // I really wish to have box pattern here :(
        (E::Split(mut branches), V::Pi(sum, closure)) => match *sum {
            V::Sum((sum_branches, telescope)) => {
                for (name, branch) in sum_branches.into_iter() {
                    let pattern_match = match branches
                        .remove(&name) {
                        Some(go) => *go,
                        None => return Err(((gamma, context), format!("Missing clause for `{}`", name))),
                    };
                    check(
                        index,
                        (Cow::Borrowed(&gamma), context.clone()),
                        pattern_match,
                        V::Pi(
                            // FIXME: using context.clone() as an alternative to telescope
                            // is a tmp workaround
                            Box::new(branch.eval(context.clone())),
                            Closure::Choice(Box::new(closure.clone()), name.clone()),
                        ),
                    )?;
                }
                if branches.is_empty() {
                    Ok((gamma, context))
                } else {
                    let clauses: Vec<_> = branches.keys().map(|br| br.as_str()).collect();
                    Err((
                        (gamma, context),
                        format!("Unexpected clauses: {}", clauses.join(" | ")),
                    ))
                }
            }
            not_sum_so_fall_through => match check_infer(
                index,
                (Cow::Borrowed(&gamma), context.clone()),
                E::Split(branches),
            )?
            .eq_normal(index, V::Pi(Box::new(not_sum_so_fall_through), closure))
            {
                Ok(()) => Ok((gamma, context)),
                Err(java) => Err(((gamma, context), java)),
            },
        },
        (expression, value) => {
            let go = (gamma, context);
            match check_infer(index, (Cow::Borrowed(&gamma), context.clone()), expression)?
                .eq_normal(index, value)
            {
                Ok(()) => Ok(go),
                Err(java) => Err((go, java)),
            }
        }
    }
}

/// To reuse code that checks if a sum type is well-typed between `check_type` and `check`
fn check_sum_type(index: u32, (gamma, context): TCS, constructors: Branch) -> TCM<TCS> {
    for constructor in constructors.values().cloned() {
        check_type(
            index,
            (Cow::Borrowed(&gamma), context.clone()),
            *constructor,
        )?;
    }
    Ok((gamma, context))
}

/// `checkMain` in Mini-TT.
pub fn check_main<'a>(expression: Expression) -> TCM<'a, TCS<'a>> {
    check_contextual(default_state(), expression)
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
pub fn check_declaration_main<'a>(declaration: Declaration) -> TCM<'a, Gamma<'a>> {
    check_declaration(0, default_state(), declaration)
}

/// To reuse code that checks if a sigma or a pi type is well-typed between `check_type` and `check`
fn check_telescoped(
    index: u32,
    (gamma, context): TCS,
    pattern: Pattern,
    first: Expression,
    second: Expression,
) -> TCM<TCS> {
    check_type(
        index,
        (Cow::Borrowed(&gamma), context.clone()),
        first.clone(),
    )?;
    let generated = generate_value(index);
    let gamma = update_gamma(
        gamma,
        &pattern,
        first.eval(context.clone()),
        generated.clone(),
    )?;
    check_type(
        index + 1,
        (gamma, up_var_rc(context, pattern, generated)),
        second,
    )
}

#[cfg(test)]
mod tests {
    use crate::syntax::Declaration;
    use crate::syntax::Expression;
    use crate::syntax::Pattern;
    use crate::type_check::check_declaration_main;
    use crate::type_check::check_main;

    #[test]
    fn simple_check() {
        check_declaration_main(Declaration::Simple(
            Pattern::Unit,
            Expression::Type,
            Expression::One,
        ))
        .unwrap();
        let error_message = check_declaration_main(Declaration::Simple(
            Pattern::Unit,
            Expression::Type,
            Expression::Unit,
        ))
        .unwrap_err();
        println!("{}", error_message);
    }

    #[test]
    fn check_pair() {
        let expr = Expression::Declaration(
            Box::new(Declaration::Simple(
                Pattern::Unit,
                Expression::One,
                Expression::Second(Box::new(Expression::Pair(
                    Box::new(Expression::Unit),
                    Box::new(Expression::Unit),
                ))),
            )),
            Box::new(Expression::Void),
        );
        check_main(expr).unwrap();
    }
}
