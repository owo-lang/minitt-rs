use crate::ast::{CaseTree, GenericBranch, Value};
use crate::check::read_back::ReadBack;
use crate::check::tcm::{TCE, TCM, TCS};

/// Check if `subtype` is the subtype of `supertype`.
pub fn check_subtype(index: u32, tcs: TCS, subtype: Value, supertype: Value) -> TCM<TCS> {
    match (subtype, supertype) {
        (Value::InferredSum(sub_tree), Value::Sum(super_tree)) =>
            check_subtype_sum(index, tcs, *sub_tree, super_tree),
        (subtype, supertype) => compare_normal(index, tcs, subtype, supertype),
    }
}

fn check_subtype_sum(index: u32, tcs: TCS, sub_tree: GenericBranch<Value>, super_tree: CaseTree) -> TCM<TCS> {
    let super_context = *super_tree.environment;
    let mut super_tree = super_tree.branches;
    for (constructor, sub_parameter) in sub_tree.into_iter() {
        let super_parameter = super_tree.remove(constructor.as_str())
            .ok_or_else(|| TCE::UnexpectedCases(constructor))?;
        // They're supposed to be well-typed, but I'm not sure.
        // A bug report is expected here.
        check_subtype(
            index,
            tcs_borrow!(tcs),
            *sub_parameter,
            super_parameter.eval(super_context.clone()),
        )?;
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
