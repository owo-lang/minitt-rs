use crate::ast::*;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;

/// `NRho` in Mini-TT, normal form telescopes (contexts).
pub type NormalTelescope = Rc<GenericTelescope<NormalExpression>>;

/// `NSClos` in Mini-TT, normal form closures.
///
/// TODO: consider replacing `Expression`
pub type NormalCaseTree = GenericCaseTree<Expression, NormalExpression>;

/// `NNeut` in Mini-TT, normal form neutral values.
pub type NormalNeutral = GenericNeutral<NormalExpression>;

/// `NExp` in Mini-TT, normal form expressions.<br/>
/// Deriving `Eq` so we can do comparison.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NormalExpression {
    Lambda(u32, Box<Self>),
    Pair(Box<Self>, Box<Self>),
    Unit,
    One,
    Type,
    Pi(Box<Self>, u32, Box<Self>),
    Sigma(Box<Self>, u32, Box<Self>),
    Constructor(String, Box<Self>),
    Split(NormalCaseTree),
    Sum(NormalCaseTree),
    InferredSum(Box<GenericBranch<NormalExpression>>),
    Neutral(NormalNeutral),
}

/// `genV` in Mini-TT.
pub fn generate_value(id: u32) -> Value {
    use crate::ast::GenericNeutral as Neutral;
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

    /// `eqNf` in Mini-TT, but returning normal forms for error reporting.<br/>
    /// Whether two structures are equivalent up to normal form.
    fn normal(index: u32, me: Self, other: Self) -> (Self::NormalForm, Self::NormalForm) {
        let me_read_back = me.read_back(index);
        let other_read_back = other.read_back(index);
        (me_read_back, other_read_back)
    }
}

impl ReadBack for Value {
    type NormalForm = NormalExpression;

    /// `rbV` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm {
        use crate::check::read_back::NormalExpression::*;
        match self {
            Value::Lambda(closure) => Lambda(
                index,
                Box::new(
                    closure
                        .instantiate(generate_value(index))
                        .read_back(index + 1),
                ),
            ),
            Value::Unit => Unit,
            Value::One => One,
            Value::Type => Type,
            Value::Pi(input, output) => Pi(
                Box::new(input.read_back(index)),
                index,
                Box::new(
                    output
                        .instantiate(generate_value(index))
                        .read_back(index + 1),
                ),
            ),
            Value::Sigma(first, second) => Sigma(
                Box::new(first.read_back(index)),
                index,
                Box::new(
                    second
                        .instantiate(generate_value(index))
                        .read_back(index + 1),
                ),
            ),
            Value::Pair(first, second) => Pair(
                Box::new(first.read_back(index)),
                Box::new(second.read_back(index)),
            ),
            Value::Constructor(name, body) => Constructor(name, Box::new(body.read_back(index))),
            Value::Split(case_tree) => Split(case_tree.read_back(index)),
            Value::Sum(constructors) => Sum(constructors.read_back(index)),
            Value::InferredSum(constructors) => {
                let mut read_back_constructors = BTreeMap::new();
                for (name, value) in constructors.into_iter() {
                    read_back_constructors.insert(name, Box::new(value.read_back(index)));
                }
                InferredSum(Box::new(read_back_constructors))
            }
            Value::Neutral(neutral) => Neutral(neutral.read_back(index)),
        }
    }
}

impl ReadBack for CaseTree {
    type NormalForm = NormalCaseTree;

    fn read_back(self, index: u32) -> Self::NormalForm {
        Self::NormalForm::boxing(*self.branches, self.environment.read_back(index))
    }
}

impl ReadBack for &TelescopeRaw {
    type NormalForm = NormalTelescope;

    /// `rbRho` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm {
        use crate::ast::GenericTelescope::*;
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
        use crate::ast::GenericNeutral::*;
        match self {
            Generated(index) => Generated(index),
            Application(function, argument) => Application(
                Box::new(function.read_back(index)),
                Box::new(argument.read_back(index)),
            ),
            First(neutral) => First(Box::new(neutral.read_back(index))),
            Second(neutral) => Second(Box::new(neutral.read_back(index))),
            Split(case_tree, body) => {
                Split(case_tree.read_back(index), Box::new(body.read_back(index)))
            }
        }
    }
}
