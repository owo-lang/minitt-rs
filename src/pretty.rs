use core::fmt::Write;
use std::fmt::{Display, Error as FmtError, Formatter};

use crate::ast::*;
use crate::check::read_back::*;

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Value::Lambda(closure) => {
                f.write_str("\u{03BB} ")?;
                closure.fmt_with_type(f, None)
            }
            Value::Pair(first, second) => write!(f, "({}, {})", first, second),
            Value::Unit => f.write_str("0"),
            Value::One => f.write_str("1"),
            Value::Pi(input, output) => {
                f.write_str("\u{03A0}")?;
                f.write_str(" ")?;
                output.fmt_with_type(f, Some(&**input))
            }
            Value::Type(level) => write!(f, "Type{}", level),
            Value::Sigma(first, second) => {
                f.write_str("\u{03A3}")?;
                f.write_str(" ")?;
                second.fmt_with_type(f, Some(&**first))
            }
            Value::Constructor(name, arguments) => write!(f, "{} {}", name, arguments),
            // Don't print context
            Value::Split(branches) => {
                f.write_str("split {")?;
                fmt_branch(branches, f)?;
                f.write_char('}')
            }
            // Don't print the context
            Value::Sum(constructors) => {
                f.write_str("Sum {")?;
                fmt_branch(constructors, f)?;
                f.write_char('}')
            }
            Value::Neutral(neutral) => write!(f, "[{}]", neutral),
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
            Expression::Lambda(pattern, parameter_type, body) => {
                f.write_str("\u{03BB} ")?;
                pattern.fmt(f)?;
                if let Some(parameter_type) = parameter_type {
                    f.write_str(": ")?;
                    parameter_type.internal.fmt(f)?;
                }
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
            Expression::Pi(input, output) => {
                f.write_str("\u{03A0}")?;
                f.write_str(" ")?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Expression::Type(level) => {
                f.write_str("Type")?;
                level.fmt(f)
            }
            Expression::Sigma(first, second) => {
                f.write_str("\u{03A3}")?;
                f.write_str(" ")?;
                first.fmt(f)?;
                f.write_str(". ")?;
                second.fmt(f)
            }
            Expression::Constructor(name, arguments) => {
                name.fmt(f)?;
                f.write_str(" ")?;
                arguments.fmt(f)
            }
            Expression::Split(clauses) => {
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
                    match &**clause {
                        Expression::Lambda(pattern, _, body) => {
                            pattern.fmt(f)?;
                            f.write_str(" => ")?;
                            body.fmt(f)
                        }
                        rest => rest.fmt(f),
                    }?;
                }
                f.write_char('}')
            }
            // Don't print the context
            Expression::Sum(constructors) => {
                f.write_str("Sum")?;
                f.write_str(" {")?;
                fmt_branch(constructors, f)?;
                f.write_char('}')
            }
            Expression::Declaration(declaration, rest) => writeln!(f, "{};\n{}", declaration, rest),
            Expression::Constant(pattern, body, rest) => {
                write!(f, "const {} = {};\n{}", pattern, body, rest)
            }
            Expression::Void => Ok(()),
            Expression::Merge(lhs, rhs) => {
                lhs.fmt(f)?;
                f.write_str(" ++ ")?;
                rhs.fmt(f)
            }
        }
    }
}

impl<Expr: Display, Value: Clone + Display> Display for GenericCase<Expr, Value> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        self.expression.fmt(f)
    }
}

impl Display for Typed {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{}: {}", self.pattern, self.expression)
    }
}

fn fmt_branch<E: Display>(branch: &GenericBranch<E>, f: &mut Formatter) -> Result<(), FmtError> {
    let mut started = false;
    for (name, clause) in branch.iter() {
        if started {
            f.write_str(" | ")?;
        } else {
            started = true;
        }
        name.fmt(f)?;
        f.write_char(' ')?;
        clause.fmt(f)?;
    }
    Ok(())
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Pattern::Pair(first, second) => write!(f, "({}, {})", first, second),
            Pattern::Unit => f.write_char('_'),
            Pattern::Var(name) => f.write_str(name.as_str()),
        }
    }
}

impl Display for Declaration {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str(if self.is_recursive { "rec" } else { "let" })?;
        f.write_char(' ')?;
        self.pattern.fmt(f)?;
        for typed in self.prefix_parameters.iter() {
            write!(f, "({})", typed)?;
        }
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
            Closure::Abstraction(_, pt, _, _) => self.fmt_with_type(f, pt.as_ref().map(|t| &**t)),
            e => e.fmt_with_type(f, None),
        }
    }
}

impl Closure {
    /// Actual implementation of `fmt` for `Closure`
    pub fn fmt_with_type(&self, f: &mut Formatter, t: Option<&Value>) -> Result<(), FmtError> {
        match self {
            Closure::Abstraction(pattern, _, body, _) => {
                pattern.fmt(f)?;
                if let Some(t) = t {
                    f.write_str(": ")?;
                    t.fmt(f)?;
                }
                f.write_str(". ")?;
                body.fmt(f)
            }
            Closure::Value(value) => value.fmt(f),
            Closure::Choice(rest, name) => write!(f, "{}. {}", name, rest),
        }
    }
}

impl<Value: Display + Clone> Display for GenericNeutral<Value> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            GenericNeutral::Generated(index) => write!(f, "<{}>", index),
            GenericNeutral::Application(function, argument) => {
                write!(f, "({} {})", function, argument)
            }
            GenericNeutral::First(pair) => write!(f, "({}.1)", pair),
            GenericNeutral::Second(pair) => write!(f, "({}.2)", pair),
            GenericNeutral::Split(clauses, argument) => {
                f.write_str("app ")?;
                argument.fmt(f)?;
                f.write_str(" {")?;
                fmt_branch(clauses, f)?;
                f.write_char('}')
            }
        }
    }
}

impl Display for NormalExpression {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        use crate::check::read_back::NormalExpression as Expression;
        match self {
            Expression::Lambda(index, expression) => {
                f.write_str("\u{03BB} <")?;
                index.fmt(f)?;
                f.write_str("> ")?;
                expression.fmt(f)
            }
            Expression::Pair(first, second) => write!(f, "({}, {})", first, second),
            Expression::Unit => f.write_str("0"),
            Expression::One => f.write_str("1"),
            Expression::Pi(input, index, output) => {
                f.write_str("\u{03A0}")?;
                write!(f, " <{}> {}. {}", index, input, output)
            }
            Expression::Type(level) => write!(f, "Type{}", level),
            Expression::Sigma(first, index, second) => {
                f.write_str("\u{03A3}")?;
                write!(f, " <{}> {}. {}", index, first, second)
            }
            Expression::Constructor(name, arguments) => write!(f, "{} {}", name, arguments),
            Expression::Split(clauses) => {
                f.write_str("split {")?;
                fmt_branch(clauses, f)?;
                f.write_char('}')
            }
            // Don't print the context
            Expression::Sum(constructors) => {
                f.write_str("Sum {")?;
                fmt_branch(constructors, f)?;
                f.write_char('}')
            }
            Expression::Neutral(neutral) => write!(f, "[{}]", neutral),
        }
    }
}

/// Actually it's for NeutralTelescope
impl<Value: Clone + Display> Display for GenericTelescope<Value> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            GenericTelescope::Nil => Ok(()),
            GenericTelescope::UpDec(previous, declaration) => {
                write!(f, "{};\n{}", declaration, previous)
            }
            GenericTelescope::UpVar(previous, pattern, value) => {
                write!(f, "var {}: {};\n{}", pattern, value, previous)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::Declaration;
    use crate::ast::Expression;
    use crate::ast::Pattern;

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
