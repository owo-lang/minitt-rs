use std::collections::BTreeMap;

use crate::ast::{Case, Value};
use crate::check::read_back::{generate_value, ReadBack};
use crate::check::tcm::{TCE, TCM, TCS};

/// Check if `subtype` is the subtype of `supertype`.
pub fn check_subtype(
    index: u32,
    tcs: TCS,
    subtype: Value,
    supertype: Value,
    read_back: bool,
) -> TCM<TCS> {
    use crate::ast::Value::*;
    match (subtype, supertype) {
        (Type(sub_level), Type(super_level)) => {
            if sub_level <= super_level {
                Ok(tcs)
            } else {
                Err(TCE::TypeMismatch(Type(sub_level), Type(super_level)))
            }
        }
        (Sum(sub_tree), Sum(super_tree)) => {
            let super_eval = |sup: Box<Case>| sup.reduce_to_value();
            let sub_eval = |sub: Box<Case>| sub.reduce_to_value();
            // TODO: this no longer need to be generic
            check_subtype_sum(index, tcs, *sub_tree, *super_tree, sub_eval, super_eval)
        }
        (Pi(sub_param, sub_closure), Pi(super_param, super_closure))
        | (Sigma(sub_param, sub_closure), Sigma(super_param, super_closure)) => {
            let tcs = check_subtype(index, tcs, *super_param, *sub_param, true)?;
            let generated = generate_value(index);
            check_subtype(
                index + 1,
                tcs_borrow!(tcs),
                sub_closure.instantiate(generated.clone()),
                super_closure.instantiate(generated),
                true,
            )?;
            Ok(tcs)
        }
        (subtype, supertype) => {
            if read_back {
                compare_normal(index, tcs, subtype, supertype)
            } else {
                Err(TCE::TypeMismatch(subtype, supertype))
            }
        }
    }
}

/// Extracted from `check_subtype` to reduce indentation (but this function requires improvements).
///
/// Usually people want to recursively call `check_subtype`, but it's gonna cause stack-overflow if
/// the sum type is recursive (like `nat`). To prevent this, I tried to call `compare_normal` before
/// recursively check.<br/>
/// I'm not sure if this recursion is actually needed.
///
/// A bug report is expected to prove this to be false.
#[inline]
pub fn check_subtype_sum<Sub, Super>(
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
        compare_normal(
            index,
            tcs_borrow!(tcs),
            sub_parameter.clone(),
            super_parameter.clone(),
        )
        .or_else(|_err| {
            check_subtype(
                index,
                tcs_borrow!(tcs),
                sub_parameter,
                super_parameter,
                false,
            )
        })?;
    }
    Ok(tcs)
}

/// Read back the type values and do syntactic comparison.
pub fn compare_normal(index: u32, tcs: TCS, subtype: Value, supertype: Value) -> TCM<TCS> {
    let (inferred_normal, expected_normal) = ReadBack::normal(index, subtype, supertype);
    if inferred_normal == expected_normal {
        Ok(tcs)
    } else {
        Err(TCE::ReadBackTypeMismatch(inferred_normal, expected_normal))
    }
}
