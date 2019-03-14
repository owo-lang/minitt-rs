use crate::ast::Value;
use crate::check::read_back::ReadBack;
use crate::check::tcm::{TCE, TCM, TCS};
use std::borrow::Cow;

pub fn check_subtype(
    index: u32,
    (gamma, context): TCS,
    subtype: Value,
    supertype: Value,
) -> TCM<TCS> {
    let (subtype, supertype) = match (subtype, supertype) {
        (Value::Sum((sub_tree, sub_context)), Value::Sum((mut super_tree, super_context))) => {
            for (constructor, sub_parameter) in sub_tree.into_iter() {
                if let Some(super_parameter) = super_tree.remove(constructor.as_str()) {
                    // They're supposed to be well-typed, but I'm not sure.
                    // A bug report is expected here.
                    check_subtype(
                        index,
                        (Cow::Borrowed(&gamma), context.clone()),
                        sub_parameter.eval(*sub_context.clone()),
                        super_parameter.eval(*super_context.clone()),
                    )?;
                } else {
                    return Err(TCE::UnexpectedCases(constructor));
                }
            }
            return Ok((gamma, context));
        }
        (subtype, supertype) => (subtype, supertype),
    };
    compare_normal(index, (gamma, context), subtype, supertype)
}

/// Read back the type values and do syntactic comparison
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
