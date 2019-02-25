use std::collections::BTreeMap;
use std::hash::Hash;

/// Virtual trait, created to simplify trait bounds for identifiers.
pub trait NameTrait: Eq + Ord + PartialOrd + Hash + Clone {}

/// `Exp` in Mini-TT.
/// Expression language for Mini-TT.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Expression<Name: NameTrait> {
    Unit,
    One,
    Type,
    Void,
    Var(Name),
    Sum(Branch<Name>),
    Function(Branch<Name>),
    Pi(Pattern<Name>, Box<Expression<Name>>, Box<Expression<Name>>),
    Sigma(Pattern<Name>, Box<Expression<Name>>, Box<Expression<Name>>),
    Lambda(Pattern<Name>, Box<Expression<Name>>),
    First(Box<Expression<Name>>),
    Second(Box<Expression<Name>>),
    Application(Box<Expression<Name>>, Box<Expression<Name>>),
    Pair(Box<Expression<Name>>, Box<Expression<Name>>),
    Constructor(Name, Box<Expression<Name>>),
    Declaration(Box<Declaration<Name>>, Box<Expression<Name>>),
}

/// Pattern matching branch.
type Branch<Name> = BTreeMap<Name, Box<Expression<Name>>>;

/// `Val` in Mini-TT, value term.
#[derive(Debug, Clone)]
pub enum Value<Name: NameTrait> {
    Lambda(Closure<Name>),
    Unit,
    One,
    Type,
    Pi(Box<Value<Name>>, Closure<Name>),
    Sigma(Box<Value<Name>>, Closure<Name>),
    Pair(Box<Value<Name>>, Box<Value<Name>>),
    Constructor(Name, Box<Value<Name>>),
    Function(SClosure<Name>),
    Sum(SClosure<Name>),
    Neutral(Neutral<Name>),
}

/// `Neut` in Mini-TT, neutral value.
#[derive(Debug, Clone)]
pub enum Neutral<Name: NameTrait> {
    Generated(u32),
    Application(Box<Neutral<Name>>, Box<Value<Name>>),
    First(Box<Neutral<Name>>),
    Second(Box<Neutral<Name>>),
    Function(Box<SClosure<Name>>, Box<Neutral<Name>>),
}

/// `Pattern` in Mini-TT.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Pattern<Name: NameTrait> {
    Pair(Box<Pattern<Name>>, Box<Pattern<Name>>),
    Unit,
    Var(Name),
}

/// `Decl` in Mini-TT.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Declaration<Name: NameTrait> {
    Simple(Pattern<Name>, Expression<Name>, Expression<Name>),
    Recursive(Pattern<Name>, Expression<Name>, Expression<Name>),
}

/// `Rho` in Mini-TT.
// TODO: replace with Vec<enum {Dec, Var}>
#[derive(Debug, Clone)]
pub enum Telescope<Name: NameTrait> {
    Nil,
    UpDec(Box<Telescope<Name>>, Declaration<Name>),
    UpVar(Box<Telescope<Name>>, Pattern<Name>, Value<Name>),
}

/// `Clos` in Mini-TT.
#[derive(Debug, Clone)]
pub enum Closure<Name: NameTrait> {
    Choice(Pattern<Name>, Expression<Name>, Box<Telescope<Name>>),
    Function(Box<Closure<Name>>, Name),
}

/// `SClos` in Mini-TT.
type SClosure<Name> = (Box<Branch<Name>>, Box<Telescope<Name>>);
