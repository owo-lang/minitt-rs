use crate::normal::*;
use crate::syntax::*;
use core::fmt::Write;
use std::fmt::{Display, Error as FmtError, Formatter};

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Value::Lambda(closure) => {
                f.write_str("\u{039B} ")?;
                closure.fmt(f)
            }
            Value::Pair(first, second) => {
                f.write_char('(')?;
                first.fmt(f)?;
                f.write_str(", ")?;
                second.fmt(f)?;
                f.write_char(')')
            }
            Value::Unit => f.write_str("0"),
            Value::One => f.write_str("1"),
            Value::Pi(input, output) => {
                f.write_str("\u{03A0} ")?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Value::Type => f.write_str("U"),
            Value::Sigma(first, second) => {
                f.write_str("\u{03A3} ")?;
                first.fmt(f)?;
                f.write_str(". ")?;
                second.fmt(f)
            }
            Value::Constructor(name, arguments) => {
                name.fmt(f)?;
                f.write_str(" ")?;
                arguments.fmt(f)
            }
            // Don't print context
            Value::Function((branches, _)) => {
                f.write_str("fun (")?;
                let mut started = false;
                for (name, clause) in branches.iter() {
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    clause.fmt(f)?;
                    if started {
                        f.write_str(", ")?;
                    } else {
                        started = true;
                    }
                }
                f.write_char(')')
            }
            // Don't print the context
            Value::Sum((constructors, _)) => {
                f.write_str("Sum (")?;
                let mut started = false;
                for (name, constructor) in constructors.iter() {
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    constructor.fmt(f)?;
                    if started {
                        f.write_str(", ")?;
                    } else {
                        started = true;
                    }
                }
                f.write_char(')')
            }
            Value::Neutral(neutral) => {
                f.write_str("#(")?;
                neutral.fmt(f)?;
                f.write_char(')')
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Expression::Var(name) => name.fmt(f),
            Expression::First(pair) => {
                f.write_char('(')?;
                pair.fmt(f)?;
                f.write_str(".1")?;
                f.write_char(')')
            }
            Expression::Second(pair) => {
                f.write_char('(')?;
                pair.fmt(f)?;
                f.write_str(".2")?;
                f.write_char(')')
            }
            Expression::Application(function, argument) => {
                f.write_char('(')?;
                function.fmt(f)?;
                f.write_char(' ')?;
                argument.fmt(f)?;
                f.write_char(')')
            }
            Expression::Lambda(pattern, expression) => {
                f.write_str("\u{039B} ")?;
                pattern.fmt(f)?;
                f.write_str(". ")?;
                expression.fmt(f)
            }
            Expression::Pair(first, second) => {
                f.write_char('(')?;
                first.fmt(f)?;
                f.write_str(", ")?;
                second.fmt(f)?;
                f.write_char(')')
            }
            Expression::Unit => f.write_str("0"),
            Expression::One => f.write_str("1"),
            Expression::Pi(pattern, input, output) => {
                pattern.fmt(f)?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Expression::Type => f.write_str("U"),
            Expression::Sigma(pattern, first, second) => {
                pattern.fmt(f)?;
                first.fmt(f)?;
                f.write_str(". ")?;
                second.fmt(f)
            }
            Expression::Constructor(name, arguments) => {
                name.fmt(f)?;
                f.write_str(" ")?;
                arguments.fmt(f)
            }
            Expression::Function(branches) => {
                f.write_str("fun (")?;
                let mut started = false;
                for (name, clause) in branches.iter() {
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    clause.fmt(f)?;
                    if started {
                        f.write_str(", ")?;
                    } else {
                        started = true;
                    }
                }
                f.write_char(')')
            }
            // Don't print the context
            Expression::Sum(constructors) => {
                f.write_str("Sum (")?;
                let mut started = false;
                for (name, constructor) in constructors.iter() {
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    constructor.fmt(f)?;
                    if started {
                        f.write_str(", ")?;
                    } else {
                        started = true;
                    }
                }
                f.write_char(')')
            }
            Expression::Declaration(declaration, rest) => {
                declaration.fmt(f)?;
                f.write_str(";\n")?;
                rest.fmt(f)
            }
            Expression::Void => Ok(()),
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Pattern::Pair(first, second) => {
                f.write_char('(')?;
                first.fmt(f)?;
                f.write_str(", ")?;
                second.fmt(f)?;
                f.write_char(')')
            }
            Pattern::Unit => f.write_char('_'),
            Pattern::Var(name) => f.write_str(name.as_str()),
        }
    }
}

impl Display for Declaration {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let (definer, pattern, signature, body) = match self {
            Declaration::Simple(pattern, signature, body) => ("let", pattern, signature, body),
            Declaration::Recursive(pattern, signature, body) => ("rec", pattern, signature, body),
        };
        f.write_str(definer)?;
        f.write_char(' ')?;
        pattern.fmt(f)?;
        f.write_str(": ")?;
        signature.fmt(f)?;
        f.write_str(" = ")?;
        body.fmt(f)
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            // Don't print the scope
            Closure::Function(pattern, expression, _) => {
                pattern.fmt(f)?;
                f.write_str(". ")?;
                expression.fmt(f)
            }
            Closure::Value(value) => value.fmt(f),
            Closure::Choice(rest, name) => {
                f.write_str(name.as_str())?;
                f.write_str(". ")?;
                rest.fmt(f)
            }
        }
    }
}

impl<Value: Display + Clone> Display for GenericNeutral<Value> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            GenericNeutral::Generated(index) => {
                f.write_char('[')?;
                index.fmt(f)?;
                f.write_char(']')
            }
            GenericNeutral::Application(function, argument) => {
                f.write_char('(')?;
                function.fmt(f)?;
                f.write_char(' ')?;
                argument.fmt(f)?;
                f.write_char(')')
            }
            GenericNeutral::First(pair) => {
                f.write_char('(')?;
                pair.fmt(f)?;
                f.write_str(".1")?;
                f.write_char(')')
            }
            GenericNeutral::Second(pair) => {
                f.write_char('(')?;
                pair.fmt(f)?;
                f.write_str(".2")?;
                f.write_char(')')
            }
            GenericNeutral::Function((clauses, _), argument) => {
                f.write_str("app (")?;
                let mut started = false;
                for (name, clause) in clauses.iter() {
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    clause.fmt(f)?;
                    if started {
                        f.write_str(", ")?;
                    } else {
                        started = true;
                    }
                }
                f.write_char(')')?;
                argument.fmt(f)
            }
        }
    }
}

impl Display for NormalExpression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        use crate::normal::NormalExpression as Expression;
        match self {
            Expression::Lambda(index, expression) => {
                f.write_str("\u{039B} [")?;
                index.fmt(f)?;
                f.write_str("] ")?;
                expression.fmt(f)
            }
            Expression::Pair(first, second) => {
                f.write_char('(')?;
                first.fmt(f)?;
                f.write_str(", ")?;
                second.fmt(f)?;
                f.write_char(')')
            }
            Expression::Unit => f.write_str("0"),
            Expression::One => f.write_str("1"),
            Expression::Pi(input, index, output) => {
                f.write_str("\u{03A0} [")?;
                index.fmt(f)?;
                f.write_str("] ")?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Expression::Type => f.write_str("U"),
            Expression::Sigma(first, index, second) => {
                f.write_str("\u{03A3} [")?;
                index.fmt(f)?;
                f.write_str("] ")?;
                first.fmt(f)?;
                f.write_str(". ")?;
                second.fmt(f)
            }
            Expression::Constructor(name, arguments) => {
                name.fmt(f)?;
                f.write_str(" ")?;
                arguments.fmt(f)
            }
            Expression::Function((clauses, _)) => {
                f.write_str("fun (")?;
                let mut started = false;
                for (name, clause) in clauses.iter() {
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    clause.fmt(f)?;
                    if started {
                        f.write_str(", ")?;
                    } else {
                        started = true;
                    }
                }
                f.write_char(')')
            }
            // Don't print the context
            Expression::Sum((constructors, _)) => {
                f.write_str("Sum (")?;
                let mut started = false;
                for (name, constructor) in constructors.iter() {
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    constructor.fmt(f)?;
                    if started {
                        f.write_str(", ")?;
                    } else {
                        started = true;
                    }
                }
                f.write_char(')')
            }
            Expression::Neutral(neutral) => {
                f.write_str("#(")?;
                neutral.fmt(f)?;
                f.write_char(')')
            }
        }
    }
}

/// Actually it's for NeutralTelescope
impl Display for GenericTelescope<NormalExpression> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str("<context>")
    }
}

#[cfg(test)]
mod tests {
    use crate::syntax::Declaration;
    use crate::syntax::Expression;
    use crate::syntax::Pattern;

    #[test]
    fn simple_expr() {
        let expr = Expression::Second(Box::new(Expression::Pair(
            Box::new(Expression::One),
            Box::new(Expression::Unit),
        )));
        println!("{}", expr);
    }

    #[test]
    fn simple_decl() {
        let var = "a".to_string();
        let expr = Expression::Declaration(
            Box::new(Declaration::Simple(
                Pattern::Unit,
                Expression::One,
                Expression::Second(Box::new(Expression::Pair(
                    Box::new(Expression::Unit),
                    Box::new(Expression::Unit),
                ))),
            )),
            Box::new(Expression::Declaration(
                Box::new(Declaration::Recursive(
                    Pattern::Var(var.clone()),
                    Expression::One,
                    Expression::First(Box::new(Expression::Pair(
                        Box::new(Expression::Unit),
                        Box::new(Expression::Var(var)),
                    ))),
                )),
                Box::new(Expression::Void),
            )),
        );
        println!("{}", expr);
    }
}
