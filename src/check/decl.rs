use crate::check::read_back::generate_value;
use crate::check::tcm::{update_gamma, Gamma, TCE, TCM, TCS};
use crate::syntax::{
    up_dec_rc, up_var_rc, AnonymousValue, Declaration, Expression, Pattern, Telescope, Typed,
};
use crate::type_check::{check, check_type};
use std::borrow::Cow;

macro_rules! try_locate {
    ($err:expr, $pattern:expr) => {
        match $err {
            TCE::Textual(s) => TCE::Located(s, $pattern.clone()),
            e => e,
        }
    };
}

/// Lift all these `parameters` into the context.<br/>
/// Returning `TCS` to reuse the variable.
///
/// `check_body` is supposed to return `signature`, `body` and `tcs`.
fn check_lift_parameters<'a>(
    index: u32,
    tcs: TCS<'a>,
    mut parameters: Vec<Typed>,
    check_body: impl FnOnce(TCS<'a>) -> TCM<(Expression, Expression, TCS<'a>)>,
) -> TCM<(Expression, Expression, TCS<'a>)> {
    if parameters.is_empty() {
        return check_body(tcs);
    }
    let (pattern, expression) = parameters.remove(0);
    let (gamma, context) = check_type(index, tcs, *expression.clone())?;
    let generated = generate_value(index);
    let type_val = expression.clone().eval(context.clone());
    let gamma = update_gamma(gamma, &pattern, type_val.clone(), generated.clone())?;

    let tcs = (gamma, up_var_rc(context, pattern.clone(), generated));
    let (signature, body, (gamma, context)) =
        check_lift_parameters(index + 1, tcs, parameters, check_body)?;

    Ok((
        Expression::Pi(
            (pattern.clone(), Box::new(*expression)),
            Box::new(signature),
        ),
        Expression::Lambda(pattern, AnonymousValue::some(type_val), Box::new(body)),
        (gamma, context),
    ))
}

/// Extracted from `checkD` in Mini-TT.<br/>
/// This part deals with recursive declarations, but without prefixed parameters.
pub fn check_recursive_declaration(
    index: u32,
    (gamma, context): TCS,
    declaration: Declaration,
) -> TCM<Gamma> {
    let pattern = declaration.pattern.clone();
    check_type(
        index,
        (Cow::Borrowed(&gamma), context.clone()),
        declaration.signature.clone(),
    )
    .map_err(|err| try_locate!(err, pattern))?;
    let signature = declaration.signature.clone().eval(context.clone());
    let generated = generate_value(index);
    let fake_gamma = update_gamma(
        Cow::Borrowed(&gamma),
        &pattern,
        signature.clone(),
        generated.clone(),
    )
    .map_err(|err| try_locate!(err, pattern))?;
    let fake_context = up_var_rc(context.clone(), pattern.clone(), generated);
    check(
        index + 1,
        (fake_gamma, fake_context),
        declaration.body.clone(),
        signature.clone(),
    )
    .map_err(|err| try_locate!(err, pattern))?;
    let body = declaration
        .body
        .clone()
        .eval(up_dec_rc(context, declaration));
    update_gamma(gamma, &pattern, signature, body).map_err(|err| try_locate!(err, pattern))
}

/// Extracted from `checkD` in Mini-TT.<br/>
/// This part deals with non-recursive declarations, but without prefixed parameters.
pub fn check_simple_declaration(
    index: u32,
    (gamma, context): TCS,
    pattern: Pattern,
    signature: Expression,
    body: Expression,
) -> TCM<Gamma> {
    check_type(
        index,
        (Cow::Borrowed(&gamma), context.clone()),
        signature.clone(),
    )
    .map_err(|err| try_locate!(err, pattern))?;
    let signature = signature.eval(context.clone());
    check(
        index,
        (Cow::Borrowed(&gamma), context.clone()),
        body.clone(),
        signature.clone(),
    )
    .map_err(|err| try_locate!(err, pattern))?;
    update_gamma(gamma, &pattern, signature, body.eval(context))
        .map_err(|err| try_locate!(err, pattern))
}

/// Originally `checkD` in Mini-TT, but now it's not because this implementation supports
/// prefixed parameters :)<br/>
/// Check if a declaration is well-typed and update the context.
pub fn check_declaration(
    index: u32,
    (gamma, context): TCS,
    declaration: Declaration,
) -> TCM<(Gamma, Option<Telescope>)> {
    use crate::syntax::DeclarationType::*;
    let tcs = (Cow::Borrowed(&*gamma), context.clone());
    if declaration.prefix_parameters.is_empty() {
        let tcs = (gamma, context);
        return match &declaration.declaration_type {
            Simple => check_simple_declaration(
                index,
                tcs,
                declaration.pattern,
                declaration.signature,
                declaration.body,
            ),
            Recursive => check_recursive_declaration(index, tcs, declaration),
        }
        .map(|gamma| (gamma, None));
    }
    let (pattern, signature, body) = match declaration {
        Declaration {
            pattern,
            prefix_parameters,
            signature,
            body,
            declaration_type: Simple,
        } => check_lift_parameters(index, tcs, prefix_parameters, |tcs| {
            let (gamma, context) = check_type(index, tcs, signature.clone())
                .map_err(|err| try_locate!(err, pattern))?;
            let tcs = check(
                index,
                (gamma, context.clone()),
                body.clone(),
                signature.clone().eval(context),
            )
            .map_err(|err| try_locate!(err, pattern))?;
            Ok((signature, body, tcs))
        })
        .map(|(signature, body, _)| (pattern, signature, body))?,
        declaration => {
            let pattern = declaration.pattern.clone();
            check_lift_parameters(index, tcs, declaration.prefix_parameters.clone(), |tcs| {
                let (gamma, context) = check_type(index, tcs, declaration.signature.clone())
                    .map_err(|err| try_locate!(err, pattern))?;
                let pattern = pattern.clone();
                let generated = generate_value(index);
                let signature = declaration.signature.clone().eval(context.clone());
                let fake_gamma = update_gamma(
                    Cow::Borrowed(&gamma),
                    &pattern,
                    signature.clone(),
                    generated.clone(),
                )
                .map_err(|err| try_locate!(err, pattern))?;
                let fake_context = up_var_rc(context.clone(), pattern.clone(), generated);
                check(
                    index + 1,
                    (fake_gamma, fake_context),
                    declaration.body.clone(),
                    signature,
                )
                .map_err(|err| try_locate!(err, pattern))?;
                Ok((
                    declaration.signature.clone(),
                    declaration.body.clone(),
                    (gamma, up_dec_rc(context, declaration)),
                ))
            })
            .map(|(signature, body, _)| (pattern, signature, body))?
        }
    };

    let body = body.eval(context.clone());
    update_gamma(
        gamma,
        &pattern,
        signature.eval(context.clone()),
        body.clone(),
    )
    .map(|gamma| (gamma, Some(up_var_rc(context, pattern.clone(), body))))
    .map_err(|err| try_locate!(err, pattern))
}
