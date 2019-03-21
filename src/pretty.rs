use crate::ast::MaybeLevel::SomeLevel;
use crate::ast::*;
use crate::check::read_back::*;
use core::fmt::Write;
use std::fmt::{Display, Error as FmtError, Formatter};

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            Value::Lambda(closure) => {
                f.write_str("\u{03BB} ")?;
                closure.fmt_with_type(f, None)
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
            Value::Pi(input, output, level) => {
                f.write_str("\u{03A0}")?;
                level.fmt(f)?;
                f.write_str(" ")?;
                output.fmt_with_type(f, Some(&**input))
            }
            Value::Type(level) => {
                f.write_str("Type")?;
                level.fmt(f)
            }
            Value::Sigma(first, second, level) => {
                f.write_str("\u{03A3}")?;
                level.fmt(f)?;
                f.write_str(" ")?;
                second.fmt_with_type(f, Some(&**first))
            }
            Value::Constructor(name, arguments) => {
                name.fmt(f)?;
                f.write_str(" ")?;
                arguments.fmt(f)
            }
            // Don't print context
            Value::Split(branches) => {
                f.write_str("split {")?;
                fmt_branch(branches, f)?;
                f.write_char('}')
            }
            // Don't print the context
            Value::Sum(constructors, level) => {
                f.write_str("Sum")?;
                level.fmt(f)?;
                f.write_str(" {")?;
                fmt_branch(constructors, f)?;
                f.write_char('}')
            }
            Value::Neutral(neutral) => {
                f.write_str("[")?;
                neutral.fmt(f)?;
                f.write_char(']')
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
            Expression::Pi((pattern, input), output, level) => {
                f.write_str("\u{03A0}")?;
                level.fmt(f)?;
                f.write_str(" ")?;
                pattern.fmt(f)?;
                f.write_str(": ")?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Expression::Type(level) => {
                f.write_str("Type")?;
                level.fmt(f)
            }
            Expression::Sigma((pattern, first), second, level) => {
                f.write_str("\u{03A3}")?;
                level.fmt(f)?;
                f.write_str(" ")?;
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
                fmt_branch(branches, f)?;
                f.write_char('}')
            }
            // Don't print the context
            Expression::Sum(constructors, level) => {
                f.write_str("Sum")?;
                level.fmt(f)?;
                f.write_str(" {")?;
                fmt_branch(constructors, f)?;
                f.write_char('}')
            }
            Expression::Declaration(declaration, rest) => {
                declaration.fmt(f)?;
                f.write_str(";\n")?;
                rest.fmt(f)
            }
            Expression::Constant(pattern, body, rest) => {
                f.write_str("const ")?;
                pattern.fmt(f)?;
                f.write_str(" = ")?;
                body.fmt(f)?;
                f.write_str(";\n")?;
                rest.fmt(f)
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
        f.write_str(if self.is_recursive { "rec" } else { "let" })?;
        f.write_char(' ')?;
        self.pattern.fmt(f)?;
        for (pattern, prefix_parameter_type) in self.prefix_parameters.iter() {
            f.write_str(" (")?;
            pattern.fmt(f)?;
            f.write_str(": ")?;
            prefix_parameter_type.fmt(f)?;
            f.write_char(')')?;
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
            Expression::Pair(first, second) => {
                f.write_char('(')?;
                first.fmt(f)?;
                f.write_str(", ")?;
                second.fmt(f)?;
                f.write_char(')')
            }
            Expression::Unit => f.write_str("0"),
            Expression::One => f.write_str("1"),
            Expression::Pi(input, index, output, level) => {
                f.write_str("\u{03A0}")?;
                level.fmt(f)?;
                f.write_str(" <")?;
                index.fmt(f)?;
                f.write_str("> ")?;
                input.fmt(f)?;
                f.write_str(". ")?;
                output.fmt(f)
            }
            Expression::Type(level) => {
                f.write_str("Type")?;
                level.fmt(f)
            }
            Expression::Sigma(first, index, second, level) => {
                f.write_str("\u{03A3}")?;
                level.fmt(f)?;
                f.write_str(" <")?;
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
            Expression::Split(clauses) => {
                f.write_str("split {")?;
                fmt_branch(clauses, f)?;
                f.write_char('}')
            }
            // Don't print the context
            Expression::Sum(constructors, level) => {
                f.write_str("Sum")?;
                level.fmt(f)?;
                f.write_str(" {")?;
                fmt_branch(constructors, f)?;
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

impl Display for MaybeLevel {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match self {
            SomeLevel(level) => level.fmt(f),
            NoLevel => f.write_str("<no_level>"),
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
