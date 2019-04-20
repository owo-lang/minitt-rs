use crate::ast::{up_dec_rc, up_var_rc, AnonymousValue, Declaration, Expression, Pattern, Typed};
use crate::check::expr::{check, check_type};
use crate::check::read_back::generate_value;
use crate::check::tcm::{update_gamma_borrow, update_gamma_lazy, Gamma, TCE, TCM, TCS};

macro_rules! try_locate {
    ($err:expr, $pattern:expr) => {
        match $err {
            TCE::Located(_, _) => $err,
            e => TCE::Located(Box::new(e), $pattern.clone()),
        }
    };
}

/// Internal state when lifting prefix-parameters.
pub type LiftState<'a> = (Expression, Expression, TCS<'a>);

/// $$
/// \frac{\rho \ \texttt{empty} \quad T:\textsf{U} \quad a:T}
///      {\textnormal{checkLiftD}(a,T,\rho)}
/// $$
/// $$
/// \frac{\rho\equiv[(v,T)::\rho'] \quad T:\textsf{U}
///       \quad v:T\vdash R:\textsf{U}
///       \quad \Gamma,v:T\vdash\textnormal{checkLiftD}(a,R,\rho')}
///      {\Gamma\vdash\textnormal{checkLiftD}(\lambda v.a,\Pi (v:T).R,\rho)}
/// $$
/// This is an extension, it's not present in Mini-TT.<br/>
/// Lift all these `parameters` into the context.<br/>
/// Returning `TCS` to reuse the variable.
///
/// `check_body` is supposed to return `signature`, `body` and `tcs`.
pub fn check_lift_parameters<'a>(
    index: u32,
    tcs: TCS<'a>,
    mut parameters: Vec<Typed>,
    check_body: impl FnOnce(TCS<'a>) -> TCM<LiftState<'a>>,
) -> TCM<LiftState<'a>> {
    if parameters.is_empty() {
        return check_body(tcs);
    }
    let parameter = parameters.remove(0);
    // Forgive me, I failed find a better name.
    let clone = parameter.clone();
    let (pattern, expression) = parameter.destruct();
    let (_, tcs) = check_type(index, tcs, expression.clone())?;
    let generated = generate_value(index);
    let type_val = expression.clone().eval(tcs.context());
    let tcs = tcs.update(pattern.clone(), type_val.clone(), generated)?;
    let (signature, body, tcs) = check_lift_parameters(index + 1, tcs, parameters, check_body)?;

    Ok((
        Expression::Pi(clone, Box::new(signature)),
        Expression::Lambda(pattern, AnonymousValue::some(type_val), Box::new(body)),
        tcs,
    ))
}

/// $$
/// \frac{\rho,\Gamma\vdash\_l A
///       \quad \Gamma\vdash p:t=[\textsf{x}\_l]\Rightarrow \Gamma\_1
///       \quad (\rho,p=[\textsf{x}\_l]),\Gamma\vdash\_{l+1} M\Leftarrow t
///       \quad \Gamma\vdash p:t=v\Rightarrow \Gamma\_2}
///      {\rho,\Gamma\vdash\_l \textsf{rec}\ p:A=M\Rightarrow \Gamma\_2}
/// $$
/// $$
/// \dbinom{t=⟦A⟧\rho,}{v=⟦M⟧(\rho,\textsf{rec}\ p:A=M)}
/// $$
/// Extracted from `checkD` in Mini-TT.<br/>
/// This part deals with recursive declarations, but without prefixed parameters.
pub fn check_recursive_declaration(index: u32, tcs: TCS, declaration: Declaration) -> TCM<Gamma> {
    let pattern = declaration.pattern.clone();
    check_type(index, tcs_borrow!(tcs), declaration.signature.clone())
        .map_err(|err| try_locate!(err, pattern))?;
    let signature = declaration.signature.clone().eval(tcs.context());
    let generated = generate_value(index);
    let fake_tcs: TCS = tcs_borrow!(tcs);
    let fake_tcs = fake_tcs
        .update(pattern.clone(), signature.clone(), generated)
        .map_err(|err| try_locate!(err, pattern))?;
    let body = declaration.body.clone();
    check(index + 1, fake_tcs, body.clone(), signature.clone())
        .map_err(|err| try_locate!(err, pattern))?;
    let context = tcs.context;
    update_gamma_lazy(tcs.gamma, &pattern, signature, || {
        body.eval(up_dec_rc(context, declaration))
    })
    .map_err(|err| try_locate!(err, pattern))
}

/// $$
/// \frac{\rho,\Gamma\vdash_l A
///       \quad \rho,\Gamma\vdash_l M\Leftarrow t
///       \quad \Gamma\vdash p:t=⟦M⟧\rho\Rightarrow \Gamma_1}
///      {\rho,\Gamma\vdash_l p:A=M\Rightarrow \Gamma_1}
/// $$
/// $$
/// (t=⟦M⟧\rho)
/// $$
/// Extracted from `checkD` in Mini-TT.<br/>
/// This part deals with non-recursive declarations, but without prefixed parameters.
pub fn check_simple_declaration(
    index: u32,
    mut tcs: TCS,
    pattern: Pattern,
    signature: Expression,
    body: Expression,
) -> TCM<Gamma> {
    let (_, new_tcs) =
        check_type(index, tcs, signature.clone()).map_err(|err| try_locate!(err, pattern))?;
    tcs = new_tcs;
    // workaround: fix error when calculate level here ↓
    let signature = signature.eval(tcs.context());
    tcs = check(index, tcs, body.clone(), signature.clone())
        .map_err(|err| try_locate!(err, pattern))?;
    let TCS { gamma, context } = tcs;
    update_gamma_lazy(gamma, &pattern, signature, || body.eval(context))
        .map_err(|err| try_locate!(err, pattern))
}

/// Originally `checkD` in Mini-TT, but now it's not because this implementation supports
/// prefixed parameters :)<br/>
/// Check if a declaration is well-typed and update the context.
pub fn check_declaration(index: u32, tcs: TCS, declaration: Declaration) -> TCM<TCS> {
    if declaration.prefix_parameters.is_empty() {
        let context = tcs.context();
        return if !declaration.is_recursive {
            check_simple_declaration(
                index,
                tcs,
                declaration.pattern.clone(),
                declaration.signature.clone(),
                declaration.body.clone(),
            )
        } else {
            check_recursive_declaration(index, tcs, declaration.clone())
        }
        .map(|gamma| TCS::new(gamma, up_dec_rc(context, declaration)));
    }
    let (pattern, signature, body) = match declaration {
        Declaration {
            pattern,
            prefix_parameters,
            signature,
            body,
            is_recursive: false,
        } => check_lift_parameters(index, tcs_borrow!(tcs), prefix_parameters, |tcs| {
            let (_, tcs) = check_type(index, tcs, signature.clone())
                .map_err(|err| try_locate!(err, pattern))?;
            let context = tcs.context();
            let tcs = check(index, tcs, body.clone(), signature.clone().eval(context))
                .map_err(|err| try_locate!(err, pattern))?;
            Ok((signature, body, tcs))
        })
        .map(|(signature, body, _)| (pattern, signature, body))?,
        declaration => {
            let pattern = declaration.pattern.clone();
            check_lift_parameters(
                index,
                tcs_borrow!(tcs),
                declaration.prefix_parameters.clone(),
                |tcs| {
                    let (_, tcs) = check_type(index, tcs, declaration.signature.clone())
                        .map_err(|err| try_locate!(err, pattern))?;
                    let pattern = pattern.clone();
                    let generated = generate_value(index);
                    let signature = declaration.signature.clone().eval(tcs.context());
                    let fake_tcs: TCS = tcs_borrow!(tcs);
                    let fake_tcs = fake_tcs
                        .update(pattern.clone(), signature.clone(), generated)
                        .map_err(|err| try_locate!(err, pattern))?;
                    check(index + 1, fake_tcs, declaration.body.clone(), signature)
                        .map_err(|err| try_locate!(err, pattern))?;
                    Ok((
                        declaration.signature.clone(),
                        declaration.body.clone(),
                        TCS::new(tcs.gamma, up_dec_rc(tcs.context, declaration)),
                    ))
                },
            )
            .map(|(signature, body, _)| (pattern, signature, body))?
        }
    };

    let TCS { gamma, context } = tcs;
    let body = body.eval(context.clone());
    update_gamma_borrow(gamma, &pattern, signature.eval(context.clone()), &body)
        .map(|gamma| TCS::new(gamma, up_var_rc(context, pattern.clone(), body)))
        .map_err(|err| try_locate!(err, pattern))
}
