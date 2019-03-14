use crate::ast::{reduce_to_value, Expression, Value};
use crate::check::read_back::{generate_value, ReadBack};
use crate::check::tcm::{TCE, TCM, TCS};
use either::Either;
use std::collections::BTreeMap;

/// Check if `subtype` is the subtype of `supertype`.
pub fn check_subtype(index: u32, tcs: TCS, subtype: Value, supertype: Value) -> TCM<TCS> {
    match (subtype, supertype) {
        (Value::Sum(sub_tree), Value::Sum(super_tree)) => {
            let (super_tree, super_environment) = super_tree.destruct();
            let (sub_tree, sub_environment) = sub_tree.destruct();
            let super_eval = |sup: Box<Either<Value, Expression>>| {
                reduce_to_value(*sup, super_environment.clone())
            };
            let sub_eval = |sub: Box<Either<Value, Expression>>| {
                reduce_to_value(*sub, sub_environment.clone())
            };
            check_subtype_sum(index, tcs, sub_tree, super_tree, sub_eval, super_eval)
        }
        (Value::Pi(sub_param, sub_closure), Value::Pi(super_param, super_closure))
        | (Value::Sigma(sub_param, sub_closure), Value::Sigma(super_param, super_closure)) => {
            let tcs = check_subtype(index, tcs, *super_param, *sub_param)?;
            let generated = generate_value(index);
            check_subtype(
                index + 1,
                tcs_borrow!(tcs),
                sub_closure.instantiate(generated.clone()),
                super_closure.instantiate(generated),
            )?;
            Ok(tcs)
        }
        (subtype, supertype) => compare_normal(index, tcs, subtype, supertype),
    }
}

#[inline]
fn check_subtype_sum<Sub, Super>(
    index: u32,
    tcs: TCS,
    sub_tree: BTreeMap<String, Sub>,
    mut super_tree: BTreeMap<String, Super>,
    sub_tree_eval: impl Fn(Sub) -> Value,
    super_tree_eval: impl Fn(Super) -> Value,
) -> TCM<TCS> {
    for (constructor, sub_parameter) in sub_tree.into_iter() {
        let super_parameter = super_tree
            .remove(constructor.as_str())
            .ok_or_else(|| TCE::UnexpectedCases(constructor))?;
        let sub_parameter = sub_tree_eval(sub_parameter);
        let super_parameter = super_tree_eval(super_parameter);
        // I'm not sure if I should recursively check or not, because if there's recursive sum type,
        // recursively `check_subtype` will cause stack-overflow.
        // A bug report is expected to prove this to be false.
        compare_normal(index, tcs_borrow!(tcs), sub_parameter, super_parameter)?;
    }
    return Ok(tcs);
}

/// Read back the type values and do syntactic comparison.
pub fn compare_normal(index: u32, tcs: TCS, subtype: Value, supertype: Value) -> TCM<TCS> {
    let (inferred_normal, expected_normal) = ReadBack::normal(index, subtype, supertype);
    if inferred_normal == expected_normal {
        Ok(tcs)
    } else {
        Err(TCE::InferredDoesNotMatchExpected(
            inferred_normal,
            expected_normal,
        ))
    }
}
