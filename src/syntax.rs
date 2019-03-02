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
    Split(Branch),
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
    Split(CaseTree),
    Sum(CaseTree),
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
    Split(GenericCaseTree<Value>, Box<Self>),
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
pub type TelescopeRc<Value> = Rc<GenericTelescope<Value>>;

/// `Rho` in Mini-TT, dependent context.
pub type Telescope = Rc<TelescopeRaw>;

/// Just for simplifying constructing an `Rc`.
pub fn up_var_rc<Value: Clone>(
    me: TelescopeRc<Value>,
    pattern: Pattern,
    value: Value,
) -> TelescopeRc<Value> {
    Rc::new(GenericTelescope::UpVar(me, pattern, value))
}

/// Just for simplifying constructing an `Rc`.
pub fn up_dec_rc<Value: Clone>(
    me: TelescopeRc<Value>,
    declaration: Declaration,
) -> TelescopeRc<Value> {
    Rc::new(GenericTelescope::UpDec(me, declaration))
}

/// Because we can't `impl` a `Default` for `Rc`.
pub fn nil_rc<Value: Clone>() -> TelescopeRc<Value> {
    Rc::new(GenericTelescope::Nil)
}

/// `Clos` in Mini-TT.
#[derive(Debug, Clone)]
pub enum Closure {
    /// `cl` in Mini-TT.<br/>
    /// Closure that does a pattern matching.
    Abstraction(Pattern, Expression, Box<Telescope>),
    /// This is not present in Mini-TT.<br/>
    /// Sometimes the closure is already an evaluated value.
    Value(Box<Value>),
    /// `clCmp` in Mini-TT.<br/>
    /// Closure that was inside of a case-split.
    Choice(Box<Self>, String),
}

/// Generic definition for two kinds of case trees
pub type GenericCaseTree<Value> = (Box<Branch>, Box<Rc<GenericTelescope<Value>>>);

/// `SClos` in Mini-TT.<br/>
/// Case tree.
pub type CaseTree = GenericCaseTree<Value>;
