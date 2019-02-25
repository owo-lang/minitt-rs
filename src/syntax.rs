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

/// `Val` in Mini-TT, value term. [`Name`] is type for identifiers
#[derive(Debug)]
pub enum Value<Name>
where
    Name: Eq,
    Name: Clone,
{
    Lambda(Closure<Name>),
    Unit,
    One,
    Type,
    Pi(Box<Value<Name>>, Closure<Name>),
    Sigma(Box<Value<Name>>, Closure<Name>),
    Pair(Box<Value<Name>>, Box<Value<Name>>),
    Constructor(Name, Box<Value<Name>>),
    Function(Box<SClosure<Name>>),
    Sum(Box<SClosure<Name>>),
    Neutral(Neutral<Name>),
}

/// `Neut` in Mini-TT, neutral value
#[derive(Debug)]
pub enum Neutral<Name>
where
    Name: Eq,
    Name: Clone,
{
    Generated(u32),
    Application(Box<Neutral<Name>>, Box<Value<Name>>),
    First(Box<Neutral<Name>>),
    Second(Box<Neutral<Name>>),
    NeutralFunction(Box<SClosure<Name>>, Box<Neutral<Name>>),
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

/// `Rho` in Mini-TT
// TODO: replace with Vec<enum {Dec, Var}>
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

/// `Clos` in Mini-TT, [`Name`] is type for identifiers
#[derive(Debug)]
pub enum Closure<Name>
where
    Name: Eq,
    Name: Clone,
{
    Choice(Pattern<Name>, Expression<Name>, Box<Telescope<Name>>),
    Function(Box<Closure<Name>>, Name),
}

/// `SClos` in Mini-TT
type SClosure<Name> = (Branch<Name>, Telescope<Name>);
