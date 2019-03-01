use std::rc::Rc;

use crate::syntax::*;

/// `NRho` in Mini-TT, normal form telescopes (contexts).
pub type NormalTelescope = Rc<GenericTelescope<NormalExpression>>;

/// `NSClos` in Mini-TT, normal form closures.
pub type NormalDeepClosure = GenericCaseTree<NormalExpression>;

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
    Function(NormalDeepClosure),
    Sum(NormalDeepClosure),
    Neutral(NormalNeutral),
}
