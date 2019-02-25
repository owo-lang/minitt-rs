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
pub type Branch<Name> = BTreeMap<Name, Box<Expression<Name>>>;

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
    Function(DeepClosure<Name>),
    Sum(DeepClosure<Name>),
    Neutral(Neutral<Name>),
}

/// Generic definition for two kinds of neutral terms
#[derive(Debug, Clone)]
pub enum GenericNeutral<Name: NameTrait, Value> {
    Generated(u32),
    Application(Box<GenericNeutral<Name, Value>>, Box<Value>),
    First(Box<GenericNeutral<Name, Value>>),
    Second(Box<GenericNeutral<Name, Value>>),
    Function(
        GenericDeepClosure<Name, Value>,
        Box<GenericNeutral<Name, Value>>,
    ),
}

/// `Neut` in Mini-TT, neutral value.
pub type Neutral<Name> = GenericNeutral<Name, Value<Name>>;

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

/// Generic definition for two kinds of telescopes
// TODO: replace with Vec<enum {Dec, Var}>
#[derive(Debug, Clone)]
pub enum GenericTelescope<Name: NameTrait, Value> {
    Nil,
    UpDec(Box<GenericTelescope<Name, Value>>, Declaration<Name>),
    UpVar(Box<GenericTelescope<Name, Value>>, Pattern<Name>, Value),
}

/// `Rho` in Mini-TT, dependent context.
pub type Telescope<Name> = GenericTelescope<Name, Value<Name>>;

/// `Clos` in Mini-TT.
#[derive(Debug, Clone)]
pub enum Closure<Name: NameTrait> {
    Choice(Pattern<Name>, Expression<Name>, Box<Telescope<Name>>),
    Function(Box<Closure<Name>>, Name),
}

/// Generic definition for two kinds of deep closures
pub type GenericDeepClosure<Name, Value> = (Box<Branch<Name>>, Box<GenericTelescope<Name, Value>>);

/// `SClos` in Mini-TT.
/// A closure that comes with a pattern, like the data type (sum) definition (all the constructors
/// are pattern-like) or the function definition (it's built on top of a pattern tree)
pub type DeepClosure<Name> = GenericDeepClosure<Name, Value<Name>>;
