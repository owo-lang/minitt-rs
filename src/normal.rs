use crate::syntax::*;

/// `NRho` in Mini-TT, normal form telescopes (contexts).
pub type NormalTelescope<Name> = GenericTelescope<Name, NormalExpression<Name>>;

/// `NSClos` in Mini-TT, normal form closures.
pub type NormalDeepClosure<Name> = GenericDeepClosure<Name, NormalExpression<Name>>;

/// `NNeut` in Mini-TT, normal form neutral values.
pub type NormalNeutral<Name> = GenericNeutral<Name, NormalExpression<Name>>;

/// `NExp` in Mini-TT, normal form expressions.
#[derive(Debug, Clone)]
pub enum NormalExpression<Name: NameTrait> {
    Lambda(u32, Box<NormalExpression<Name>>),
    Pair(Box<NormalExpression<Name>>, Box<NormalExpression<Name>>),
    Unit,
    One,
    Type,
    Pi(
        Box<NormalExpression<Name>>,
        u32,
        Box<NormalExpression<Name>>,
    ),
    Sigma(
        Box<NormalExpression<Name>>,
        u32,
        Box<NormalExpression<Name>>,
    ),
    Constructor(Name, Box<NormalExpression<Name>>),
    Function(NormalDeepClosure<Name>),
    Sum(NormalDeepClosure<Name>),
    Neutral(NormalNeutral<Name>),
}
