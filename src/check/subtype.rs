use crate::ast::Value;
use crate::check::read_back::ReadBack;
use crate::check::tcm::{TCE, TCM, TCS};

pub fn is_subtype(index: u32, tcs: TCS, subtype: Value, supertype: Value) -> TCM<TCS> {
    // TODO: subtyping
    compare_normal(index, tcs, subtype, supertype)
}

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
