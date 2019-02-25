use crate::normal::*;
use crate::reduce::*;
use crate::syntax::*;

trait ReadBack {
    type NormalForm;

    /// Interface for `rbV`, `rbN` and `rbRho` in Mini-TT.
    fn read_back(self) -> Self::NormalForm;
}

impl<Name: DebuggableNameTrait> ReadBack for Value<Name> {
    type NormalForm = NormalExpression<Name>;

    /// `rbV` in Mini-TT.
    fn read_back(self) -> Self::NormalForm {
        unimplemented!()
    }
}

impl<Name: DebuggableNameTrait> ReadBack for Telescope<Name> {
    type NormalForm = NormalTelescope<Name>;

    /// `rbRho` in Mini-TT.
    fn read_back(self) -> Self::NormalForm {
        use crate::syntax::GenericTelescope::*;
        match self {
            Nil => Nil,
            UpDec(context, declaration) => UpDec(Box::new(context.read_back()), declaration),
            UpVar(context, pattern, val) => {
                UpVar(Box::new(context.read_back()), pattern, val.read_back())
            }
        }
    }
}

impl<Name: DebuggableNameTrait> ReadBack for Neutral<Name> {
    type NormalForm = NormalNeutral<Name>;

    /// `rbN` in Mini-TT.
    fn read_back(self) -> Self::NormalForm {
        use crate::syntax::GenericNeutral::*;
        match self {
            Generated(index) => Generated(index),
            Application(function, argument) => Application(
                Box::new(function.read_back()),
                Box::new(argument.read_back()),
            ),
            First(neutral) => First(Box::new(neutral.read_back())),
            Second(neutral) => Second(Box::new(neutral.read_back())),
            Function((case_tree, context), body) => Function(
                (case_tree, Box::new(context.read_back())),
                Box::new(body.read_back()),
            ),
        }
    }
}
