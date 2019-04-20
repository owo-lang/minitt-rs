use super::{check_declaration_main, check_main};
use crate::ast::{Declaration, Expression, Pattern};

#[test]
fn simple_check() {
    check_declaration_main(Declaration::simple(
        Pattern::Unit,
        vec![],
        Expression::Type(0),
        Expression::One,
    ))
    .unwrap();
    let error_message = check_declaration_main(Declaration::simple(
        Pattern::Unit,
        vec![],
        Expression::Type(0),
        Expression::Unit,
    ))
    .unwrap_err();
    println!("{}", error_message);
}

#[test]
fn check_pair() {
    let expr = Expression::Declaration(
        Box::new(Declaration::simple(
            Pattern::Unit,
            vec![],
            Expression::One,
            Expression::Second(Box::new(Expression::Pair(
                Box::new(Expression::Unit),
                Box::new(Expression::Unit),
            ))),
        )),
        Box::new(Expression::Void),
    );
    check_main(expr).unwrap();
}
