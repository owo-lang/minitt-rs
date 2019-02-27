use crate::syntax::*;
use crate::normal::*;
use core::fmt::Write;
use std::fmt::{Display, Error as FmtError, Formatter};

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        unimplemented!()
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        unimplemented!()
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
        unimplemented!()
    }
}

impl Display for NormalExpression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        unimplemented!()
    }
}

/// Actually it's for NeutralTelescope
impl Display for GenericTelescope<NormalExpression> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        unimplemented!()
    }
}
