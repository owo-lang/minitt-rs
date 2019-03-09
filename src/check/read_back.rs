use crate::check::normal::*;
use crate::syntax::*;
use std::fmt::Display;
use std::rc::Rc;

/// `genV` in Mini-TT.
pub fn generate_value(id: u32) -> Value {
    use crate::syntax::GenericNeutral as Neutral;
    Value::Neutral(Neutral::Generated(id))
}

/// Since all of `Value`, `Neutral` and `Telescope` have a read back function,
/// I extracted this common interface for them.
///
/// Implementing `Sized` to make the compiler happy.
pub trait ReadBack: Sized {
    /// Corresponding normal form type for the read-backable structures.<br/>
    /// This is needed because Rust does not support Higher-Kinded Types :(
    type NormalForm: Eq + Display;

    /// Interface for `rbV`, `rbN` and `rbRho` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm;

    /// Sometimes you don't want to pass an argument, and let me do this for you :)
    fn read_back_please(self) -> Self::NormalForm {
        self.read_back(0)
    }

    /// `eqNf` in Mini-TT.<br/>
    /// Whether two structures are equivalent up to normal form.
    fn eq_normal(self, index: u32, other: Self) -> Result<(), String> {
        let self_read_back = self.read_back(index);
        let other_read_back = other.read_back(index);
        if self_read_back == other_read_back {
            Ok(())
        } else {
            Err(format!(
                "Type-Check: {} is not equal to {} up to normal form.",
                self_read_back, other_read_back
            ))
        }
    }
}

impl ReadBack for Value {
    type NormalForm = NormalExpression;

    /// `rbV` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm {
        match self {
            Value::Lambda(closure) => NormalExpression::Lambda(
                index,
                Box::new(
                    closure
                        .instantiate(generate_value(index))
                        .read_back(index + 1),
                ),
            ),
            Value::Unit => NormalExpression::Unit,
            Value::One => NormalExpression::One,
            Value::Type => NormalExpression::Type,
            Value::Pi(input, output) => NormalExpression::Pi(
                Box::new(input.read_back(index)),
                index,
                Box::new(
                    output
                        .instantiate(generate_value(index))
                        .read_back(index + 1),
                ),
            ),
            Value::Sigma(first, second) => NormalExpression::Sigma(
                Box::new(first.read_back(index)),
                index,
                Box::new(
                    second
                        .instantiate(generate_value(index))
                        .read_back(index + 1),
                ),
            ),
            Value::Pair(first, second) => NormalExpression::Pair(
                Box::new(first.read_back(index)),
                Box::new(second.read_back(index)),
            ),
            Value::Constructor(name, body) => {
                NormalExpression::Constructor(name, Box::new(body.read_back(index)))
            }
            Value::Split((case_tree, context)) => {
                NormalExpression::Split((case_tree, Box::new(context.read_back(index))))
            }
            Value::Sum((constructors, context)) => {
                NormalExpression::Sum((constructors, Box::new(context.read_back(index))))
            }
            Value::Neutral(neutral) => NormalExpression::Neutral(neutral.read_back(index)),
        }
    }
}

impl ReadBack for &TelescopeRaw {
    type NormalForm = NormalTelescope;

    //noinspection RsBorrowChecker
    /// `rbRho` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm {
        use crate::syntax::GenericTelescope::*;
        match self {
            Nil => Rc::new(Nil),
            UpDec(context, declaration) => {
                Rc::new(UpDec(context.read_back(index), declaration.clone()))
            }
            UpVar(context, pattern, val) => Rc::new(UpVar(
                context.read_back(index),
                pattern.clone(),
                val.clone().read_back(index),
            )),
        }
    }
}

impl ReadBack for Neutral {
    type NormalForm = NormalNeutral;

    /// `rbN` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm {
        use crate::syntax::GenericNeutral::*;
        match self {
            Generated(index) => Generated(index),
            Application(function, argument) => Application(
                Box::new(function.read_back(index)),
                Box::new(argument.read_back(index)),
            ),
            First(neutral) => First(Box::new(neutral.read_back(index))),
            Second(neutral) => Second(Box::new(neutral.read_back(index))),
            Split((case_tree, context), body) => Split(
                (case_tree, Box::new(context.read_back(index))),
                Box::new(body.read_back(index)),
            ),
        }
    }
}
