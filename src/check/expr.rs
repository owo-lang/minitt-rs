use std::cmp::max;
use std::collections::BTreeMap;

use either::Either;

use super::decl::check_declaration;
use super::read_back::generate_value;
use super::subtype::check_subtype;
use super::tcm::{update_gamma, TCE, TCM, TCS};
use crate::ast::{Branch, Closure, Expression, GenericCase, Level, Typed, Value};

/// $$
/// \frac{\Gamma(x)\rightarrow t}
///      {\rho,\Gamma\vdash\_l x\Rightarrow t}
/// \quad
/// \frac{\rho,\Gamma\vdash\_l M \Rightarrow \Pi\ t\ g
///       \quad \rho,\Gamma\vdash\_l N\Leftarrow t}
///      {\rho,\Gamma\vdash\_l M\ N \Rightarrow \textsf{inst}\ g(⟦N⟧\rho)}
/// $$
/// $$
/// \frac{}{\rho,\Gamma\vdash\_l 0 \Rightarrow \textbf{1}}
/// \quad
/// \frac{}{\rho,\Gamma\vdash\_l \textbf{1} \Rightarrow \textsf{U}}
/// $$
/// $$
/// \frac{\rho,\Gamma\vdash\_l M\Rightarrow \Sigma\ t\ g}
///      {\rho,\Gamma\vdash\_l M.1\Rightarrow t}
/// \quad
/// \frac{\rho,\Gamma\vdash\_l M\Rightarrow \Sigma\ t\ g}
///      {\rho,\Gamma\vdash\_l M.2 \Rightarrow \textsf{inst}\ g((⟦M⟧\rho).1)}
/// $$
/// `checkI` in Mini-TT.<br/>
/// Type inference rule. More inferences are added here (maybe it's useful?).
pub fn check_infer(index: u32, mut tcs: TCS, expression: Expression) -> TCM<Value> {
    use crate::ast::Expression::*;
    // change for below usage of `expression`
    match expression.clone() {
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
            let inferred = check_infer(index, tcs, *expression)?;
            let case = GenericCase::new(Either::Left(inferred), context);
            map.insert(name, Box::new(case));
            Ok(Value::Sum(map))
        }
        Pair(left, right) => {
            let left = check_infer(index, tcs_borrow!(tcs), *left)?;
            let right = check_infer(index, tcs_borrow!(tcs), *right)?;
            let right = Closure::Value(Box::new(right));
            Ok(Value::Sigma(Box::new(left), right))
        }
        First(pair) => match check_infer(index, tcs, *pair)? {
            Value::Sigma(first, _) => Ok(*first),
            e => Err(TCE::WantSigmaBut(e)),
        },
        Second(pair) => {
            let TCS { gamma, context } = tcs;
            match check_infer(index, TCS::new(gamma, context.clone()), *pair.clone())? {
                Value::Sigma(_, second) => Ok(second.instantiate(pair.eval(context).first())),
                e => Err(TCE::WantSigmaBut(e)),
            }
        }
        Sum(branches) => {
            let mut max_level = 0;
            for (_, branch) in branches.into_iter() {
                let (level, new) = check_type(index, tcs, *branch)?;
                tcs = new;
                if level > max_level {
                    max_level = level;
                }
            }
            Ok(Value::Type(max_level))
        }
        Merge(left, right) => {
            if left.clone().eval_to_sum(tcs.context()).is_none() {
                return Err(TCE::WantSumBut(Either::Right(*left)));
            }
            if right.clone().eval_to_sum(tcs.context()).is_none() {
                return Err(TCE::WantSumBut(Either::Right(*right)));
            }
            let left_level = match check_infer(index, tcs_borrow!(tcs), *left)? {
                Value::Type(left_level) => left_level,
                e => return Err(TCE::WantSumBut(Either::Left(e))),
            };
            let right_level = match check_infer(index, tcs_borrow!(tcs), *right)? {
                Value::Type(right_level) => right_level,
                e => return Err(TCE::WantSumBut(Either::Left(e))),
            };
            Ok(Value::Type(max(left_level, right_level)))
        }
        Pi(input, output) | Sigma(input, output) => {
            let (left_level, new) = check_type(index, tcs, *input.expression.clone())?;
            tcs = new;
            let input_type = input.expression.eval(tcs.context());
            let generated = generate_value(index);
            let gamma = update_gamma(tcs.gamma, &input.pattern, input_type, generated)?;
            let (right_level, _) = check_type(index + 1, TCS::new(gamma, tcs.context), *output)?;
            // Does this need to depend on the level of the return type?
            Ok(Value::Type(max(left_level, right_level)))
        }
        Application(function, argument) => match *function {
            Lambda(pattern, Some(parameter_type), return_value) => {
                let parameter_type = *parameter_type.internal;
                tcs = check(index, tcs, *argument, parameter_type.clone())?;
                let generated = generate_value(index + 1);
                let tcs = tcs.update(pattern, parameter_type, generated)?;
                check_infer(index + 1, tcs, *return_value)
            }
            f => match check_infer(index, tcs_borrow!(tcs), f)? {
                Value::Pi(input, output) => {
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

/// $$
/// \frac{}{\rho,\Gamma\vdash_l \textsf{U}}
/// $$
/// $$
/// \frac{\rho,\Gamma\vdash_l A
///       \quad \Gamma\vdash p:⟦A⟧\rho=[\textsf{x}\_l]\Rightarrow\Gamma_1
///       \quad (\rho,p=[\textsf{x}\_l]), \Gamma\_1\vdash\_{l+1}B}
///      {\rho,\Gamma\vdash_l \Pi\ p:A.B}
/// $$
/// $$
/// \frac{\rho,\Gamma\vdash_l A
///       \quad \Gamma\vdash p:⟦A⟧\rho=[\textsf{x}\_l]\Rightarrow\Gamma_1
///       \quad (\rho,p=[\textsf{x}\_l]), \Gamma\_1\vdash\_{l+1}B}
///      {\rho,\Gamma\vdash_l \Sigma\ p:A.B}
/// $$
/// $$
/// \frac{\rho,\Gamma\vdash_l A\Leftarrow\textsf{U}}
///      {\rho,\Gamma\vdash_l A}
/// \textnormal{(if other rules are not applicable)}
/// $$
/// `checkT` in Mini-TT.<br/>
/// Check if an expression is a well-typed type expression.
pub fn check_type(index: u32, tcs: TCS, expression: Expression) -> TCM<(Level, TCS)> {
    use crate::ast::Expression::*;
    match expression {
        Sum(constructors) => check_sum_type(index, tcs, constructors),
        Pi(first, second) | Sigma(first, second) => check_telescoped(index, tcs, first, *second),
        Merge(left, right) => check_merge_type(index, tcs, *left, *right),
        Type(level) => Ok((level + 1, tcs)),
        Void | One => Ok((0, tcs)),
        expression => {
            let inferred = check_infer(index, tcs_borrow!(tcs), expression)?;
            match inferred.level_safe() {
                Some(level) if level > 0 => Ok((level - 1, tcs)),
                _ => Err(TCE::NotTypeType(inferred)),
            }
        }
    }
}

/// To reuse code that checks if a merge expression is well-typed between `check_type` and `check`
pub fn check_merge_type(
    index: u32,
    mut tcs: TCS,
    left: Expression,
    right: Expression,
) -> TCM<(Level, TCS)> {
    let (left_level, new_tcs) = check_type(index, tcs, left.clone())?;
    tcs = new_tcs;
    let (right_level, new_tcs) = check_type(index, tcs, right.clone())?;
    tcs = new_tcs;
    let left_branches = match left.clone().eval_to_sum(tcs.context()) {
        Some(branches) => branches,
        None => return Err(TCE::WantSumBut(Either::Right(left))),
    };
    let right_branches = match right.clone().eval_to_sum(tcs.context()) {
        Some(branches) => branches,
        None => return Err(TCE::WantSumBut(Either::Right(right))),
    };
    for left_branch in left_branches.into_iter() {
        if right_branches.contains(&left_branch) {
            return Err(TCE::DuplicateBranch(left_branch));
        }
    }
    Ok((max(left_level, right_level), tcs))
}

/// $$
/// \frac{\rho,\Gamma\vdash\_l M\Leftarrow ⟦A\_i⟧v}
///      {\rho,\Gamma\vdash\_l c\_i \ M \Leftarrow \textsf{Sum}
///       \lang c\_1\ A\_1 | \dots | c\_n\ A\_n,v \rang}
/// $$
/// $$
/// \frac{}{\rho,\Gamma\vdash\_l 0 \Leftarrow \textbf{1}}
/// \quad
/// \frac{}{\rho,\Gamma\vdash\_l \textbf{1} \Leftarrow \textsf{U}}
/// $$
/// $$
/// \frac{\rho,\Gamma\vdash_l M\_1\Leftarrow \Pi(⟦A\_1⟧v)(g \circ c\_1)
///       \dots
///       \rho,\Gamma\vdash_l M\_n\Leftarrow \Pi(⟦A\_n⟧v)(g \circ c\_n)}
///      {\rho,\Gamma\vdash\_l \textsf{fun}(c\_1\rightarrow M\_1 | \dots | c\_n \rightarrow M\_n)
///       \Leftarrow \Pi(\textsf{Sum}\lang c\_1:A\_1 | \dots | c\_n:A\_n,v \rang)g}
/// $$
/// $$
/// \frac{\rho,\Gamma\vdash\_l D\Rightarrow \Gamma\_1
///       \quad (\rho,\Gamma),\Gamma\_1\vdash\_l M\Leftarrow t}
///      {\rho,\Gamma\vdash\_l D; M\Leftarrow t}
/// $$
/// $$
/// \dots (\textnormal{There are too many, please checkout the original paper for more})
/// $$
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
        (E::Lambda(pattern, _, body), V::Pi(signature, closure)) => {
            let fake_tcs: TCS = tcs_borrow!(tcs);
            let generated = generate_value(index);
            let fake_tcs = fake_tcs.update(pattern, *signature, generated.clone())?;
            check(index + 1, fake_tcs, *body, closure.instantiate(generated))?;
            Ok(tcs)
        }
        (E::Pair(first, second), V::Sigma(first_type, second_type)) => {
            tcs = check(index, tcs, *first.clone(), *first_type)?;
            let context = tcs.context();
            check(
                index,
                tcs,
                *second,
                second_type.instantiate(first.eval(context)),
            )
        }
        (E::Constructor(name, body), V::Sum(constructors)) => {
            let constructor = constructors
                .get(&name)
                .ok_or_else(|| TCE::InvalidConstructor(name))?
                .clone()
                .reduce_to_value();
            check(index, tcs, *body, constructor)
        }
        (E::Sum(constructors), V::Type(level)) => {
            check_level(level, check_sum_type(index, tcs, constructors)?)
        }
        (E::Merge(left, right), V::Type(level)) => {
            check_level(level, check_merge_type(index, tcs, *left, *right)?)
        }
        (E::Sigma(first, second), V::Type(level)) | (E::Pi(first, second), V::Type(level)) => {
            check_level(level, check_telescoped(index, tcs, first, *second)?)
        }
        (E::Declaration(declaration, rest), rest_type) => {
            let tcs = check_declaration(index, tcs, *declaration)?;
            check(index, tcs, *rest, rest_type)
        }
        (E::Constant(pattern, body, rest), rest_type) => {
            let signature = check_infer(index, tcs_borrow!(tcs), *body.clone())?;
            let body_val = body.eval(tcs.context());
            let tcs = tcs.update(pattern, signature, body_val)?;
            check(index, tcs, *rest, rest_type)
        }
        // I really wish to have box pattern here :(
        (E::Split(mut branches), V::Pi(sum, closure)) => match *sum {
            V::Sum(sum_branches) => {
                for (name, branch) in sum_branches.into_iter() {
                    let pattern_match = match branches.remove(&name) {
                        Some(pattern_match) => *pattern_match,
                        None => return Err(TCE::MissingCase(name)),
                    };
                    let branch_value = branch.reduce_to_value();
                    let signature = V::Pi(
                        Box::new(branch_value),
                        Closure::Choice(Box::new(closure.clone()), name.clone()),
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
                V::Pi(Box::new(not_sum_so_fall_through), closure),
            ),
        },
        (expression, value) => check_fallback(index, tcs, expression, value),
    }
}

/// $$
/// \frac{i < j}{\Gamma\vdash \textsf{U}\_i <: \textsf{U}\_j}
/// $$
/// Level comparison
pub fn check_level(level: u32, (actual_level, tcs): (u32, TCS)) -> TCM<TCS> {
    if actual_level <= level {
        Ok(tcs)
    } else {
        Err(TCE::LevelMismatch(actual_level, level))
    }
}

/// $$
/// \frac{\rho,\Gamma\vdash\_l M \Rightarrow t'
///       \quad \textsf{R}\_l t = \textsf{R}\_l t'}
///      {\rho,\Gamma\vdash\_l M\Leftarrow t}
/// $$
/// Fallback rule of instance check.<br/>
/// First infer the expression type, then do subtyping comparison.
pub fn check_fallback(index: u32, tcs: TCS, body: Expression, signature: Value) -> TCM<TCS> {
    let inferred = check_infer(index, tcs_borrow!(tcs), body)?;
    check_subtype(index, tcs, inferred, signature, true)
}

/// $$
/// \frac{\rho,\Gamma\vdash\_l A\_1\Leftarrow \textsf{U}
///       \dots
///       \rho,\Gamma\vdash\_l A\_n\Leftarrow \textsf{U}}
///      {\rho,\Gamma\vdash\_l
///       \textsf{Sum}(c\_1\ A\_1|\dots|c\_n\ A\_n)\Leftarrow \textsf{U}}
/// $$
/// To reuse code that checks if a sum type is well-typed between `check_type` and `check`
pub fn check_sum_type(index: u32, mut tcs: TCS, constructors: Branch) -> TCM<(Level, TCS)> {
    let mut max_level = 0;
    for constructor in constructors.values().cloned() {
        let (level, new) = check_type(index, tcs, *constructor)?;
        tcs = new;
        if level > max_level {
            max_level = level;
        }
    }
    Ok((max_level, tcs))
}

/// $$
/// \frac{\rho,\Gamma\vdash_l A
///       \quad \Gamma\vdash p:⟦A⟧\rho=[\textsf{x}\_l]\Rightarrow\Gamma_1
///       \quad (\rho,p=[\textsf{x}\_l]), \Gamma\_1\vdash\_{l+1}B}
///      {\rho,\Gamma\vdash\_l (\Pi /\Sigma) \ p:A.B}
/// $$
/// To reuse code that checks if a sigma or a pi type is well-typed between `check_type` and `check`
pub fn check_telescoped(
    index: u32,
    mut tcs: TCS,
    first: Typed,
    second: Expression,
) -> TCM<(Level, TCS)> {
    let (_, new) = check_type(index, tcs, *first.expression.clone())?;
    tcs = new;
    let generated = generate_value(index);
    let internal_tcs = tcs_borrow!(tcs).update(
        first.pattern,
        first.expression.eval(tcs.context()),
        generated,
    )?;
    let (level, _) = check_type(index + 1, internal_tcs, second)?;
    Ok((level, tcs))
}
