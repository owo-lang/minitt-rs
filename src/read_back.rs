use crate::normal::*;
use crate::reduce::*;
use crate::syntax::*;

/// `genV` in Mini-TT.
pub fn generate_value<Name: NameTrait>(id: u32) -> Value<Name> {
    use crate::syntax::GenericNeutral as Neutral;
    Value::Neutral(Neutral::Generated(id))
}

trait ReadBack {
    type NormalForm;

    /// Interface for `rbV`, `rbN` and `rbRho` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm;
}

impl<Name: DebuggableNameTrait> ReadBack for Value<Name> {
    type NormalForm = NormalExpression<Name>;

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
            Value::Pi(first, second) => NormalExpression::Pi(
                Box::new(first.read_back(index)),
                index,
                Box::new(
                    second
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
            Value::Function((case_tree, context)) => {
                NormalExpression::Function((case_tree, Box::new(context.read_back(index))))
            }
            Value::Sum((constructors, context)) => {
                NormalExpression::Sum((constructors, Box::new(context.read_back(index))))
            }
            Value::Neutral(neutral) => NormalExpression::Neutral(neutral.read_back(index)),
        }
    }
}

impl<Name: DebuggableNameTrait> ReadBack for Telescope<Name> {
    type NormalForm = NormalTelescope<Name>;

    /// `rbRho` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm {
        use crate::syntax::GenericTelescope::*;
        match self {
            Nil => Nil,
            UpDec(context, declaration) => UpDec(Box::new(context.read_back(index)), declaration),
            UpVar(context, pattern, val) => UpVar(
                Box::new(context.read_back(index)),
                pattern,
                val.read_back(index),
            ),
        }
    }
}

impl<Name: DebuggableNameTrait> ReadBack for Neutral<Name> {
    type NormalForm = NormalNeutral<Name>;

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
            Function((case_tree, context), body) => Function(
                (case_tree, Box::new(context.read_back(index))),
                Box::new(body.read_back(index)),
            ),
        }
    }
}
