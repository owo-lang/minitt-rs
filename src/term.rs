/// Expression language, [`Name`] is type for identifiers
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Expression<Name>
where
    Name: Eq,
    Name: Clone,
{
    Unit,
    One,
    Type,
    Void,
    Var(Name),
    Sum(Branch<Name>),
    Fun(Branch<Name>),
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

type Branch<Name> = Vec<(Name, Box<Expression<Name>>)>;

/// Value term, [`Name`] is type for identifiers
#[derive(Debug)]
pub enum Value<Name>
where
    Name: Eq,
    Name: Clone,
{
    // TODO: fun, sum, neut
    Lambda(Closure<Name>),
    Unit,
    One,
    Type,
    Pi(Box<Value<Name>>, Closure<Name>),
    Sigma(Box<Value<Name>>, Closure<Name>),
    Pair(Box<Value<Name>>, Box<Value<Name>>),
    Constructor(Name, Box<Value<Name>>),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Pattern<Name>
where
    Name: Eq,
    Name: Clone,
{
    Pair(Box<Pattern<Name>>, Box<Pattern<Name>>),
    Unit,
    Var(Name),
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Declaration<Name>
where
    Name: Eq,
    Name: Clone,
{
    Simple(Pattern<Name>, Expression<Name>, Expression<Name>),
    Recursive(Pattern<Name>, Expression<Name>, Expression<Name>),
}

#[derive(Debug)]
pub enum Telescope<Name>
where
    Name: Eq,
    Name: Clone,
{
    Nil,
    UpDec(Box<Telescope<Name>>, Declaration<Name>),
    UpVar(Box<Telescope<Name>>, Pattern<Name>, Value<Name>),
}

/// Closure, [`Name`] is type for identifiers
#[derive(Debug)]
pub enum Closure<Name>
where
    Name: Eq,
    Name: Clone,
{
    Choice(Pattern<Name>, Expression<Name>, Box<Telescope<Name>>),
    Function(Box<Closure<Name>>, Name),
}

/// Simple term types
pub type SimpleTerm = Value<String>;
/// Term types for testing
pub type TestTerm = Value<u32>;
