use crate::normal::*;
use crate::syntax::*;
use core::fmt::Write;
use std::fmt::{Display, Error as FmtError, Formatter};

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Value::Lambda(closure) => {
                f.write_str("\u{03BB} ")?;
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
            Value::Split((branches, _)) => {
                f.write_str("split {")?;
                let mut started = false;
                for (name, clause) in branches.iter() {
                    if started {
                        f.write_str(" | ")?;
                    } else {
                        started = true;
                    }
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    clause.fmt(f)?;
                }
                f.write_char('}')
            }
            // Don't print the context
            Value::Sum((constructors, _)) => {
                f.write_str("sum {")?;
                let mut started = false;
                for (name, constructor) in constructors.iter() {
                    if started {
                        f.write_str(" | ")?;
                    } else {
                        started = true;
                    }
                    name.fmt(f)?;
                    f.write_str(": ")?;
                    constructor.fmt(f)?;
                }
                f.write_char('}')
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
            Expression::Lambda(pattern, body) => {
                f.write_str("\u{03BB} ")?;
                pattern.fmt(f)?;
                f.write_str(". ")?;
                body.fmt(f)
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
                f.write_str("\u{03A0} ")?;
                pattern.fmt(f)?;
                f.write_str(": ")?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Expression::Type => f.write_str("U"),
            Expression::Sigma(pattern, first, second) => {
                f.write_str("\u{03A3} ")?;
                pattern.fmt(f)?;
                f.write_str(": ")?;
                first.fmt(f)?;
                f.write_str(". ")?;
                second.fmt(f)
            }
            Expression::Constructor(name, arguments) => {
                name.fmt(f)?;
                f.write_str(" ")?;
                arguments.fmt(f)
            }
            Expression::Split(branches) => {
                f.write_str("split {")?;
                let mut started = false;
                for (name, clause) in branches.iter() {
                    if started {
                        f.write_str(" | ")?;
                    } else {
                        started = true;
                    }
                    name.fmt(f)?;
                    f.write_char(' ')?;
                    clause.fmt(f)?;
                }
                f.write_char('}')
            }
            // Don't print the context
            Expression::Sum(constructors) => {
                f.write_str("sum {")?;
                let mut started = false;
                for (name, constructor) in constructors.iter() {
                    if started {
                        f.write_str(" | ")?;
                    } else {
                        started = true;
                    }
                    name.fmt(f)?;
                    f.write_char(' ')?;
                    constructor.fmt(f)?;
                }
                f.write_char('}')
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
        f.write_str(match self.declaration_type {
            DeclarationType::Simple => "let",
            DeclarationType::Recursive => "rec",
        })?;
        f.write_char(' ')?;
        self.pattern.fmt(f)?;
        f.write_str(": ")?;
        self.signature.fmt(f)?;
        f.write_str(" = ")?;
        self.body.fmt(f)
    }
}

impl Display for Closure {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            // Don't print the scope
            Closure::Abstraction(pattern, expression, _) => {
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
                f.write_char('<')?;
                index.fmt(f)?;
                f.write_char('>')
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
            GenericNeutral::Split((clauses, _), argument) => {
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
                f.write_str("\u{03BB} <")?;
                index.fmt(f)?;
                f.write_str("> ")?;
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
                f.write_str("\u{03A0} <")?;
                index.fmt(f)?;
                f.write_str("> ")?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Expression::Type => f.write_str("U"),
            Expression::Sigma(first, index, second) => {
                f.write_str("\u{03A3} <")?;
                index.fmt(f)?;
                f.write_str("> ")?;
                first.fmt(f)?;
                f.write_str(". ")?;
                second.fmt(f)
            }
            Expression::Constructor(name, arguments) => {
                name.fmt(f)?;
                f.write_str(" ")?;
                arguments.fmt(f)
            }
            Expression::Split((clauses, _)) => {
                f.write_str("split {")?;
                let mut started = false;
                for (name, clause) in clauses.iter() {
                    if started {
                        f.write_str(" | ")?;
                    } else {
                        started = true;
                    }
                    name.fmt(f)?;
                    f.write_char(' ')?;
                    clause.fmt(f)?;
                }
                f.write_char('}')
            }
            // Don't print the context
            Expression::Sum((constructors, _)) => {
                f.write_str("sum {")?;
                let mut started = false;
                for (name, constructor) in constructors.iter() {
                    if started {
                        f.write_str(" | ")?;
                    } else {
                        started = true;
                    }
                    name.fmt(f)?;
                    f.write_char(' ')?;
                    constructor.fmt(f)?;
                }
                f.write_char('}')
            }
            Expression::Neutral(neutral) => {
                f.write_str("[")?;
                neutral.fmt(f)?;
                f.write_char(']')
            }
        }
    }
}

/// Actually it's for NeutralTelescope
impl<Value: Clone + Display> Display for GenericTelescope<Value> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            GenericTelescope::Nil => Ok(()),
            GenericTelescope::UpDec(previous, declaration) => {
                declaration.fmt(f)?;
                f.write_str(";\n")?;
                previous.fmt(f)
            }
            GenericTelescope::UpVar(previous, pattern, value) => {
                f.write_str("var ")?;
                pattern.fmt(f)?;
                f.write_str(": ")?;
                value.fmt(f)?;
                f.write_str(";\n")?;
                previous.fmt(f)
            }
        }
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
            Box::new(Declaration::simple(
                Pattern::Unit,
                vec![],
                Expression::One,
                Expression::Second(Box::new(Expression::Pair(
                    Box::new(Expression::Unit),
                    Box::new(Expression::Unit),
                ))),
            )),
            Box::new(Expression::Declaration(
                Box::new(Declaration::recursive(
                    Pattern::Var(var.clone()),
                    vec![],
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
