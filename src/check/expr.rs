use std::borrow::Cow;

use crate::ast::{up_var_rc, Branch, Closure, Expression, Pattern, Value};
use crate::check::decl::check_declaration;
use crate::check::read_back::{generate_value, ReadBack};
use crate::check::tcm::{update_gamma, update_gamma_borrow, TCE, TCM, TCS};

/// `checkI` in Mini-TT.<br/>
/// Type inference rule. More inferences are added here (maybe it's useful?).
pub fn check_infer(index: u32, (gamma, context): TCS, expression: Expression) -> TCM<Value> {
    use crate::ast::Expression::*;
    match expression {
        Unit => Ok(Value::One),
        Type | Void | One => Ok(Value::Type),
        Var(name) => gamma
            .get(&name)
            .cloned()
            .ok_or_else(|| TCE::UnresolvedName(name)),
        Pair(left, right) => {
            let left = check_infer(index, (Cow::Borrowed(&gamma), context.clone()), *left)?;
            let right = check_infer(index, (Cow::Borrowed(&gamma), context.clone()), *right)?;
            Ok(Value::Sigma(
                Box::new(left),
                Closure::Value(Box::new(right)),
            ))
        }
        First(pair) => match check_infer(index, (gamma, context), *pair)? {
            Value::Sigma(first, _) => Ok(*first),
            e => TCE::default_error(format!("Expected Sigma, got: `{}`.", e)),
        },
        Pi((pattern, input), output) | Sigma((pattern, input), output) => {
            let (gamma, context) = check_type(index, (gamma, context), *input.clone())?;
            let input_type = input.eval(context.clone());
            let generated = generate_value(index);
            let gamma = update_gamma(gamma, &pattern, input_type, generated)?;
            check_type(index + 1, (gamma, context), *output)?;
            Ok(Value::Type)
        }
        Second(pair) => match check_infer(index, (gamma, context.clone()), *pair.clone())? {
            Value::Sigma(_, second) => Ok(second.instantiate(pair.eval(context).first())),
            e => TCE::default_error(format!("Expected Sigma, got: `{}`.", e)),
        },
        Application(function, argument) => match *function {
            Lambda(pattern, Some(parameter_type), return_value) => {
                let parameter_type = *parameter_type.internal;
                let tcs = (Cow::Borrowed(&*gamma), context.clone());
                check(index, tcs, *argument, parameter_type.clone())?;
                let generated = generate_value(index + 1);
                let gamma = update_gamma_borrow(gamma, &pattern, parameter_type, &generated)?;
                let context = up_var_rc(context, pattern, generated);
                check_infer(index + 1, (gamma, context), *return_value)
            }
            f => match check_infer(index, (Cow::Borrowed(&gamma), context.clone()), f)? {
                Value::Pi(input, output) => {
                    check(index, (gamma, context.clone()), *argument.clone(), *input)?;
                    Ok(output.instantiate(argument.eval(context)))
                }
                e => TCE::default_error(format!(
                    "Expected Pi, got `{}` (argument: `{}`).",
                    e, argument
                )),
            },
        },
        e => Err(TCE::CannotInfer(e)),
    }
}

/// `checkT` in Mini-TT.<br/>
/// Check if an expression is a well-typed type expression.
pub fn check_type(index: u32, tcs: TCS, expression: Expression) -> TCM<TCS> {
    use crate::ast::Expression::*;
    match expression {
        Sum(constructors) => check_sum_type(index, tcs, constructors),
        Pi((pattern, first), second) | Sigma((pattern, first), second) => {
            check_telescoped(index, tcs, pattern, *first, *second)
        }
        Type | Void | One => Ok(tcs),
        expression => check(index, tcs, expression, Value::Type),
    }
}

/// `check` in Mini-TT.<br/>
/// However, telescope and gamma are preserved for REPL use.
pub fn check(index: u32, (gamma, context): TCS, expression: Expression, value: Value) -> TCM<TCS> {
    use crate::ast::Expression as E;
    use crate::ast::Value as V;
    match (expression, value) {
        (E::Unit, V::One) | (E::Type, V::Type) | (E::One, V::Type) => Ok((gamma, context)),
        // There's nothing left to check.
        (E::Void, _) => Ok((gamma, context)),
        (E::Lambda(pattern, _, body), V::Pi(signature, closure)) => {
            let generated = generate_value(index);
            let gamma = update_gamma_borrow(gamma, &pattern, *signature, &generated)?;
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
            let constructor = *constructors
                .get(&name)
                .ok_or_else(|| TCE::InvalidConstructor(name))?
                .clone();
            check(index, (gamma, context), *body, constructor.eval(*telescope))
        }
        (E::Sum(constructors), V::Type) => check_sum_type(index, (gamma, context), constructors),
        (E::Sigma((pattern, first), second), V::Type)
        | (E::Pi((pattern, first), second), V::Type) => {
            check_telescoped(index, (gamma, context), pattern, *first, *second)
        }
        (E::Declaration(declaration, rest), rest_type) => {
            let (gamma, context) =
                check_declaration(index, (gamma, context.clone()), *declaration)?;
            check(index, (gamma, context), *rest, rest_type)
        }
        (E::Constant(pattern, body, rest), rest_type) => {
            let signature = check_infer(
                index,
                (Cow::Borrowed(&gamma), context.clone()),
                *body.clone(),
            )?;
            let body_val = body.eval(context.clone());
            let gamma = update_gamma_borrow(gamma, &pattern, signature, &body_val)?;
            let context = up_var_rc(context, pattern, body_val);
            check(index, (gamma, context), *rest, rest_type)
        }
        // I really wish to have box pattern here :(
        (E::Split(mut branches), V::Pi(sum, closure)) => match *sum {
            V::Sum((sum_branches, telescope)) => {
                for (name, branch) in sum_branches.into_iter() {
                    let pattern_match = match branches.remove(&name) {
                        Some(pattern_match) => *pattern_match,
                        None => return Err(TCE::MissingCase(name)),
                    };
                    check(
                        index,
                        (Cow::Borrowed(&gamma), context.clone()),
                        pattern_match,
                        V::Pi(
                            Box::new(branch.eval(*telescope.clone())),
                            Closure::Choice(Box::new(closure.clone()), name.clone()),
                        ),
                    )?;
                }
                if branches.is_empty() {
                    Ok((gamma, context))
                } else {
                    let clauses: Vec<_> = branches.keys().map(|br| br.as_str()).collect();
                    Err(TCE::UnexpectedCases(clauses.join(" | ")))
                }
            }
            not_sum_so_fall_through => check_normal(
                index,
                (gamma, context),
                E::Split(branches),
                V::Pi(Box::new(not_sum_so_fall_through), closure),
            ),
        },
        (expression, value) => check_normal(index, (gamma, context), expression, value),
    }
}

fn check_normal(index: u32, (gamma, context): TCS, body: Expression, signature: Value) -> TCM<TCS> {
    let inferred = check_infer(index, (Cow::Borrowed(&gamma), context.clone()), body)?;
    let (inferred_normal, expected_normal) = ReadBack::normal(index, inferred, signature);
    if inferred_normal == expected_normal {
        Ok((gamma, context))
    } else {
        Err(TCE::InferredDoesNotMatchExpected(
            inferred_normal,
            expected_normal,
        ))
    }
}

/// To reuse code that checks if a sum type is well-typed between `check_type` and `check`
pub fn check_sum_type(index: u32, (gamma, context): TCS, constructors: Branch) -> TCM<TCS> {
    for constructor in constructors.values().cloned() {
        check_type(
            index,
            (Cow::Borrowed(&gamma), context.clone()),
            *constructor,
        )?;
    }
    Ok((gamma, context))
}

/// To reuse code that checks if a sigma or a pi type is well-typed between `check_type` and `check`
pub fn check_telescoped(
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
