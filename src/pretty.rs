use crate::normal::*;
use crate::syntax::*;
use core::fmt::Write;
use std::fmt::{Display, Error as FmtError, Formatter};

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str("TODO")
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Expression::Var(name) => name.fmt(f),
            Expression::First(pair) => {
                pair.fmt(f)?;
                f.write_str(".1")
            }
            Expression::Second(pair) => {
                pair.fmt(f)?;
                f.write_str(".2")
            }
            Expression::Application(function, argument) => {
                f.write_char('(')?;
                function.fmt(f)?;
                f.write_char(' ')?;
                argument.fmt(f)?;
                f.write_char(')')
            }
            Expression::Lambda(index, expression) => {
                f.write_char('[')?;
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
            _ => unimplemented!(),
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Pattern::Pair(first, second) => {
                first.fmt(f)?;
                f.write_str(", ")?;
                second.fmt(f)
            }
            Pattern::Unit => f.write_char(' '),
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
        body.fmt(f)?;
        f.write_str(";\n")
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
            Closure::Choice(rest, name) => {
                f.write_str("\u{039B} ")?;
                f.write_str(name.as_str())?;
                f.write_str(". ")?;
                rest.fmt(f)
            }
        }
    }
}

impl<Value: Display + Clone> Display for GenericNeutral<Value> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str("TODO")
    }
}

impl Display for NormalExpression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        use crate::normal::NormalExpression as Expression;
        match self {
            Expression::Lambda(index, expression) => {
                f.write_char('[')?;
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
                f.write_str("n(")?;
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
