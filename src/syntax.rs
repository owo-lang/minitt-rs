use std::collections::BTreeMap;
use std::rc::Rc;

/// `Exp` in Mini-TT.
/// Expression language for Mini-TT.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Expression {
    Unit,
    One,
    Type,
    Void,
    Var(String),
    Sum(Branch),
    Function(Branch),
    Pi(Pattern, Box<Self>, Box<Self>),
    Sigma(Pattern, Box<Self>, Box<Self>),
    Lambda(Pattern, Box<Self>),
    First(Box<Self>),
    Second(Box<Self>),
    Application(Box<Self>, Box<Self>),
    Pair(Box<Self>, Box<Self>),
    Constructor(String, Box<Self>),
    Declaration(Box<Declaration>, Box<Self>),
}

/// Pattern matching branch.
pub type Branch = BTreeMap<String, Box<Expression>>;

/// `Val` in Mini-TT, value term.
#[derive(Debug, Clone)]
pub enum Value {
    Lambda(Closure),
    Unit,
    One,
    Type,
    Pi(Box<Self>, Closure),
    Sigma(Box<Self>, Closure),
    Pair(Box<Self>, Box<Self>),
    Constructor(String, Box<Self>),
    Function(DeepClosure),
    Sum(DeepClosure),
    Neutral(Neutral),
}

/// Generic definition for two kinds of neutral terms.
///
/// Implementing `Eq` because of `NormalExpression`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GenericNeutral<Value: Clone> {
    Generated(u32),
    Application(Box<Self>, Box<Value>),
    First(Box<Self>),
    Second(Box<Self>),
    Function(GenericDeepClosure<Value>, Box<Self>),
}

/// `Neut` in Mini-TT, neutral value.
pub type Neutral = GenericNeutral<Value>;

/// `Patt` in Mini-TT.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Pattern {
    Pair(Box<Pattern>, Box<Pattern>),
    Unit,
    Var(String),
}

/// `Decl` in Mini-TT.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Declaration {
    Simple(Pattern, Expression, Expression),
    Recursive(Pattern, Expression, Expression),
}

impl Declaration {
    pub fn pattern(&self) -> &Pattern {
        use Declaration::*;
        match self {
            Simple(pattern, _, _) => pattern,
            Recursive(pattern, _, _) => pattern,
        }
    }
}

/// Generic definition for two kinds of telescopes.<br/>
/// `Value` can be specialized with `Value` or `NormalExpression`.
///
/// Implementing `Eq` because of `NormalExpression`
// TODO: replace with Vec<enum {Dec, Var}> maybe?
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GenericTelescope<Value: Clone> {
    Nil,
    UpDec(Rc<Self>, Declaration),
    UpVar(Rc<Self>, Pattern, Value),
}

pub type TelescopeRaw = GenericTelescope<Value>;

/// `Rho` in Mini-TT, dependent context.
pub type Telescope = Rc<TelescopeRaw>;

/// Just for simplifying constructing an `Rc`.
pub fn up_var_rc<Value: Clone>(
    me: Rc<GenericTelescope<Value>>,
    pattern: Pattern,
    value: Value,
) -> Rc<GenericTelescope<Value>> {
    Rc::new(GenericTelescope::UpVar(me, pattern, value))
}

/// Just for simplifying constructing an `Rc`.
pub fn up_dec_rc<Value: Clone>(
    me: Rc<GenericTelescope<Value>>,
    declaration: Declaration,
) -> Rc<GenericTelescope<Value>> {
    Rc::new(GenericTelescope::UpDec(me, declaration))
}

pub fn nil_rc<Value: Clone>() -> Rc<GenericTelescope<Value>> {
    Rc::new(GenericTelescope::Nil)
}

/// `Clos` in Mini-TT.
#[derive(Debug, Clone)]
pub enum Closure {
    /// `cl` in Mini-TT.<br/>
    /// Closure that does a pattern matching.
    Function(Pattern, Expression, Box<Telescope>),
    /// This is not present in Mini-TT.<br/>
    /// Sometimes the closure is already an evaluated value.
    Value(Box<Value>),
    /// `clCmp` in Mini-TT.
    /// Closure that has an extra lambda abstraction.
    Choice(Box<Self>, String),
}

/// Generic definition for two kinds of deep closures
pub type GenericDeepClosure<Value> = (Box<Branch>, Box<Rc<GenericTelescope<Value>>>);

/// `SClos` in Mini-TT.<br/>
/// A closure that comes with a pattern, like the data type (sum) definition (all the constructors
/// are pattern-like) or the function definition (it's built on top of a pattern tree)
pub type DeepClosure = GenericDeepClosure<Value>;
