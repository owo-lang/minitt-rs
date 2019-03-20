use crate::ast::{
    reduce_to_value, up_var_rc, Branch, Closure, Expression, GenericCaseTree, Pattern, Value,
};
use crate::check::decl::check_declaration;
use crate::check::read_back::generate_value;
use crate::check::subtype::check_subtype;
use crate::check::tcm::{update_gamma, update_gamma_borrow, TCE, TCM, TCS};
use either::Either;
use std::collections::BTreeMap;
use crate::ast::MaybeLevel::SomeLevel;

/// `checkI` in Mini-TT.<br/>
/// Type inference rule. More inferences are added here (maybe it's useful?).
pub fn check_infer(index: u32, mut tcs: TCS, expression: Expression) -> TCM<Value> {
    use crate::ast::Expression::*;
    match expression {
        Unit => Ok(Value::One),
        Type(level) => Ok(Value::Type(level + 1)),
        Void | One => Ok(Value::Type(0)),
        Var(name) => tcs
            .gamma
            .get(&name)
            .cloned()
            .ok_or_else(|| TCE::UnresolvedName(name)),
        Constructor(name, expression) => {
            let mut map = BTreeMap::new();
            let context = tcs.context.clone();
            let inferred = Either::Left(check_infer(index, tcs, *expression)?);
            map.insert(name, Box::new(inferred));
            Ok(Value::Sum(GenericCaseTree::boxing(map, context), 0))
        }
        Pair(left, right) => {
            let left = check_infer(index, tcs_borrow!(tcs), *left)?;
            let right = check_infer(index, tcs_borrow!(tcs), *right)?;
            let right = Closure::Value(Box::new(right));
            Ok(Value::Sigma(Box::new(left), right, 0))
        }
        First(pair) => match check_infer(index, tcs, *pair)? {
            Value::Sigma(first, _, _) => Ok(*first),
            e => Err(TCE::WantSigmaBut(e)),
        },
        Second(pair) => {
            let TCS { gamma, context } = tcs;
            match check_infer(index, TCS::new(gamma, context.clone()), *pair.clone())? {
                Value::Sigma(_, second, _) => Ok(second.instantiate(pair.eval(context).first())),
                e => Err(TCE::WantSigmaBut(e)),
            }
        }
        Sum(branches, _) => {
            let mut max = 0u32;
            for (_name, branch) in branches.into_iter() {
                let (level, new) = check_type(index, tcs, *branch)?;
                tcs = new;
                if level > max {
                    max = level;
                }
            }
            Ok(Value::Type(max))
        }
        Pi((pattern, input), output, _) | Sigma((pattern, input), output, _) => {
            let (level, tcs) = check_type(index, tcs, *input.clone())?;
            let input_type = input.eval(tcs.context());
            let generated = generate_value(index);
            let gamma = update_gamma(tcs.gamma, &pattern, input_type, generated)?;
            check_type(index + 1, TCS::new(gamma, tcs.context), *output)?;
            // Does this need to depend on the level of the return type?
            Ok(Value::Type(level))
        }
        Application(function, argument) => match *function {
            Lambda(pattern, Some(parameter_type), return_value) => {
                let parameter_type = *parameter_type.internal;
                tcs = check(index, tcs, *argument, parameter_type.clone())?;
                let generated = generate_value(index + 1);
                let gamma = update_gamma_borrow(tcs.gamma, &pattern, parameter_type, &generated)?;
                let context = up_var_rc(tcs.context, pattern, generated);
                check_infer(index + 1, TCS::new(gamma, context), *return_value)
            }
            f => match check_infer(index, tcs_borrow!(tcs), f)? {
                Value::Pi(input, output, _) => {
                    let context = tcs.context();
                    check(index, tcs, *argument.clone(), *input)?;
                    Ok(output.instantiate(argument.eval(context)))
                }
                e => Err(TCE::WantPiBut(e, *argument)),
            },
        },
        Declaration(_, _) | Constant(_, _, _) => Err(tce_unreachable!()),
        e => Err(TCE::CannotInfer(e)),
    }
}

/// `checkT` in Mini-TT.<br/>
/// Check if an expression is a well-typed type expression.
pub fn check_type(index: u32, tcs: TCS, expression: Expression) -> TCM<(u32, TCS)> {
    use crate::ast::Expression::*;
    match expression {
        Sum(constructors, level) => check_sum_type(index, tcs, constructors),
        Pi((pattern, first), second, level) | Sigma((pattern, first), second, level) => {
            check_telescoped(index, tcs, pattern, *first, *second)
        }
        Type(level) => Ok((level + 1, tcs)),
        Void | One => Ok((0, tcs)),
        expression => {
            let inferred = check_infer(index, tcs_borrow!(tcs), expression)?;
            match inferred.level_safe() {
                SomeLevel(level) if level > 0 => Ok((level - 1, tcs)),
                _ => Err(TCE::NotTypeType(inferred))
            }
        }
    }
}

/// `check` in Mini-TT.<br/>
/// However, telescope and gamma are preserved for REPL use.
pub fn check(index: u32, mut tcs: TCS, expression: Expression, value: Value) -> TCM<TCS> {
    use crate::ast::Expression as E;
    use crate::ast::Value as V;
    match (expression, value) {
        (E::Unit, V::One) | (E::One, V::Type(0)) => Ok(tcs),
        (E::Type(low), V::Type(high)) => {
            if low < high {
                Ok(tcs)
            } else {
                Err(TCE::TypeMismatch(V::Type(low + 1), V::Type(high)))
            }
        }
        // There's nothing left to check.
        (E::Void, _) => Ok(tcs),
        (E::Lambda(pattern, _, body), V::Pi(signature, closure, _)) => {
            let TCS { gamma, context } = tcs_borrow!(tcs);
            let generated = generate_value(index);
            let gamma = update_gamma_borrow(gamma, &pattern, *signature, &generated)?;
            check(
                index + 1,
                TCS::new(gamma, up_var_rc(context, pattern, generated.clone())),
                *body,
                closure.instantiate(generated),
            )?;
            Ok(tcs)
        }
        (E::Pair(first, second), V::Sigma(first_type, second_type, _)) => {
            tcs = check(index, tcs, *first.clone(), *first_type)?;
            let context = tcs.context();
            check(
                index,
                tcs,
                *second,
                second_type.instantiate(first.eval(context)),
            )
        }
        (E::Constructor(name, body), V::Sum(constructors, _)) => {
            let constructor = *constructors
                .branches
                .get(&name)
                .ok_or_else(|| TCE::InvalidConstructor(name))?
                .clone();
            let constructor_type = reduce_to_value(constructor, *constructors.environment);
            check(index, tcs, *body, constructor_type)
        }
        (E::Sum(constructors, _), V::Type(level)) => {
            check_level(level, check_sum_type(index, tcs, constructors)?)
        }
        (E::Sigma((pattern, first), second, _), V::Type(level))
        | (E::Pi((pattern, first), second, _), V::Type(level)) => check_level(
            level,
            check_telescoped(index, tcs, pattern, *first, *second)?,
        ),
        (E::Declaration(declaration, rest), rest_type) => {
            let tcs = check_declaration(index, tcs, *declaration)?;
            check(index, tcs, *rest, rest_type)
        }
        (E::Constant(pattern, body, rest), rest_type) => {
            let signature = check_infer(index, tcs_borrow!(tcs), *body.clone())?;
            let TCS { gamma, context } = tcs_borrow!(tcs);
            let body_val = body.eval(context.clone());
            let gamma = update_gamma_borrow(gamma, &pattern, signature, &body_val)?;
            let context = up_var_rc(context, pattern, body_val);
            check(index, TCS::new(gamma, context), *rest, rest_type)?;
            Ok(tcs)
        }
        // I really wish to have box pattern here :(
        (E::Split(mut branches), V::Pi(sum, closure, _)) => match *sum {
            V::Sum(sum_branches, _) => {
                for (name, branch) in sum_branches.branches.into_iter() {
                    let pattern_match = match branches.remove(&name) {
                        Some(pattern_match) => *pattern_match,
                        None => return Err(TCE::MissingCase(name)),
                    };
                    let signature = V::Pi(
                        Box::new(reduce_to_value(*branch, *sum_branches.environment.clone())),
                        Closure::Choice(Box::new(closure.clone()), name.clone()),
                        0,
                    );
                    tcs = check(index, tcs, pattern_match, signature)?;
                }
                if branches.is_empty() {
                    Ok(tcs)
                } else {
                    let clauses: Vec<_> = branches.keys().map(|br| br.as_str()).collect();
                    Err(TCE::UnexpectedCases(clauses.join(" | ")))
                }
            }
            not_sum_so_fall_through => check_fallback(
                index,
                tcs,
                E::Split(branches),
                V::Pi(Box::new(not_sum_so_fall_through), closure, 0),
            ),
        },
        (expression, value) => check_fallback(index, tcs, expression, value),
    }
}

/// Level comparison
pub fn check_level(level: u32, (actual_level, tcs): (u32, TCS)) -> TCM<TCS> {
    if actual_level <= level {
        Ok(tcs)
    } else {
        Err(TCE::LevelMismatch(actual_level, level))
    }
}

/// Fallback rule of instance check.<br/>
/// First infer the expression type, then do subtyping comparison.
pub fn check_fallback(index: u32, tcs: TCS, body: Expression, signature: Value) -> TCM<TCS> {
    let inferred = check_infer(index, tcs_borrow!(tcs), body)?;
    check_subtype(index, tcs, inferred, signature, true)
}

/// To reuse code that checks if a sum type is well-typed between `check_type` and `check`
pub fn check_sum_type(index: u32, mut tcs: TCS, constructors: Branch) -> TCM<(u32, TCS)> {
    let mut max = 0;
    for constructor in constructors.values().cloned() {
        let (level, new) = check_type(index, tcs, *constructor)?;
        tcs = new;
        if level > max {
            max = level
        }
    }
    Ok((max, tcs))
}

/// To reuse code that checks if a sigma or a pi type is well-typed between `check_type` and `check`
pub fn check_telescoped(
    index: u32,
    mut tcs: TCS,
    pattern: Pattern,
    first: Expression,
    second: Expression,
) -> TCM<(u32, TCS)> {
    let (_, new) = check_type(index, tcs, first.clone())?;
    tcs = new;
    let TCS { gamma, context } = tcs_borrow!(tcs);
    let generated = generate_value(index);
    let gamma = update_gamma(
        gamma,
        &pattern,
        first.eval(context.clone()),
        generated.clone(),
    )?;
    let (level, _) = check_type(
        index + 1,
        TCS::new(gamma, up_var_rc(context, pattern, generated)),
        second,
    )?;
    Ok((level, tcs))
}
