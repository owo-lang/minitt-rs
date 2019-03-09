use std::collections::BTreeMap;
use std::rc::Rc;

/// `Exp` in Mini-TT.
/// Expression language for Mini-TT.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expression {
    /// `0`
    Unit,
    /// `1`
    One,
    /// `U`
    Type,
    /// Empty file
    Void,
    /// `bla`
    Var(String),
    /// `sum { Bla x }`
    Sum(Branch),
    /// `split { Bla x => y }`
    Split(Branch),
    /// `\Pi a: b. c`
    Pi(Typed, Box<Self>),
    /// `\Sigma a: b. c`
    Sigma(Typed, Box<Self>),
    /// `\lambda a. c`, the optional value is the type of the argument.<br/>
    /// This cannot be specified during parsing because it's used for generated intermediate values
    /// during type-checking.
    Lambda(Pattern, Option<AnonymousValue>, Box<Self>),
    /// `bla.1`
    First(Box<Self>),
    /// `bla.2`
    Second(Box<Self>),
    /// `f a`
    Application(Box<Self>, Box<Self>),
    /// `a, b`
    Pair(Box<Self>, Box<Self>),
    /// `Cons a`
    Constructor(String, Box<Self>),
    /// `const bla`, this is an extension: a declaration whose type-signature is inferred.
    /// This is very similar to a `Declaration`.
    Constant(Pattern, Box<Self>, Box<Self>),
    /// `let bla` or `rec bla`
    Declaration(Box<Declaration>, Box<Self>),
}

/// Just a wrapper for a value but does not do `Eq` comparison.
/// This is an implementation detail and should not be noticed much when reading the source code.
#[derive(Debug, Clone)]
pub struct AnonymousValue {
    pub internal: Box<Value>,
}

impl AnonymousValue {
    pub fn new(value: Value) -> AnonymousValue {
        AnonymousValue {
            internal: Box::new(value),
        }
    }

    pub fn some(value: Value) -> Option<AnonymousValue> {
        Some(Self::new(value))
    }
}

impl Eq for AnonymousValue {}
impl PartialEq<AnonymousValue> for AnonymousValue {
    fn eq(&self, _other: &AnonymousValue) -> bool {
        true
    }
}

/// Pattern matching branch.
pub type Branch = BTreeMap<String, Box<Expression>>;

/// Pattern with type explicitly specified
pub type Typed = (Pattern, Box<Expression>);

/// `Val` in Mini-TT, value term.<br/>
/// Terms are either of canonical form or neutral form.
#[derive(Debug, Clone)]
pub enum Value {
    /// Canonical form: lambda abstraction.
    Lambda(Closure),
    /// Canonical form: unit instance.
    Unit,
    /// Canonical form: unit type.
    One,
    /// Canonical form: type universe.
    Type,
    /// Canonical form: pi type (type for dependent functions).
    Pi(Box<Self>, Closure),
    /// Canonical form: sigma type (type for dependent pair).
    Sigma(Box<Self>, Closure),
    /// Canonical form: Pair value (value for sigma).
    Pair(Box<Self>, Box<Self>),
    /// Canonical form: call to a constructor.
    Constructor(String, Box<Self>),
    /// Canonical form: case-split.
    Split(CaseTree),
    /// Canonical form: sum type.
    Sum(CaseTree),
    /// Neutral form.
    Neutral(Neutral),
}

/// Generic definition for two kinds of neutral terms.
///
/// Implementing `Eq` because of `NormalExpression`
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GenericNeutral<Value: Clone> {
    /// Neutral form: stuck on a free variable.
    Generated(u32),
    /// Neutral form: stuck on applying on a free variable.
    Application(Box<Self>, Box<Value>),
    /// Neutral form: stuck on trying to find the first element of a free variable.
    First(Box<Self>),
    /// Neutral form: stuck on trying to find the second element of a free variable.
    Second(Box<Self>),
    /// Neutral form: stuck on trying to case-split a free variable.
    Split(GenericCaseTree<Value>, Box<Self>),
}

/// `Neut` in Mini-TT, neutral value.
pub type Neutral = GenericNeutral<Value>;

/// `Patt` in Mini-TT.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Pattern {
    /// Pair pattern. This sounds like trivial and useless, but we can achieve mutual recursion by
    /// using this pattern.
    Pair(Box<Pattern>, Box<Pattern>),
    /// Unit pattern, used for introducing anonymous definitions.
    Unit,
    /// Variable name pattern, the most typical pattern.
    Var(String),
}

/// Whether a type is recursive or not.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum DeclarationType {
    Simple,
    Recursive,
}

/// `Decl` in Mini-TT.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Declaration {
    pub pattern: Pattern,
    pub prefix_parameters: Vec<Typed>,
    pub signature: Expression,
    pub body: Expression,
    pub declaration_type: DeclarationType,
}

impl Declaration {
    /// Constructor
    pub fn new(
        pattern: Pattern,
        prefix_parameters: Vec<Typed>,
        signature: Expression,
        body: Expression,
        declaration_type: DeclarationType,
    ) -> Self {
        Self {
            pattern,
            prefix_parameters,
            signature,
            body,
            declaration_type,
        }
    }

    /// Non-recursive declarations
    pub fn simple(
        pattern: Pattern,
        prefix_parameters: Vec<Typed>,
        signature: Expression,
        body: Expression,
    ) -> Self {
        Self::new(
            pattern,
            prefix_parameters,
            signature,
            body,
            DeclarationType::Simple,
        )
    }

    /// Recursive declarations
    pub fn recursive(
        pattern: Pattern,
        prefix_parameters: Vec<Typed>,
        signature: Expression,
        body: Expression,
    ) -> Self {
        Self::new(
            pattern,
            prefix_parameters,
            signature,
            body,
            DeclarationType::Recursive,
        )
    }
}

/// Generic definition for two kinds of telescopes.<br/>
/// `Value` can be specialized with `Value` or `NormalExpression`.
///
/// Implementing `Eq` because of `NormalExpression`
// TODO: replace with Vec<enum {Dec, Var}> maybe?
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GenericTelescope<Value: Clone> {
    /// Empty telescope
    Nil,
    /// In Mini-TT, checked declarations are put here. However, it's not possible to store a
    /// recursive declaration as an `Expression` (which is a member of `Declaration`) here.
    ///
    /// The problem is quite complicated and can be reproduced by checking out 0.1.5 revision and
    /// type-check this code:
    ///
    /// ```ignore
    /// rec nat : U = sum { Zero 1 | Suc nat };
    /// -- Inductive definition of nat
    ///
    /// let one : nat = Zero 0;
    /// let two : nat = Suc one;
    /// -- Unresolved reference
    /// ```
    UpDec(Rc<Self>, Declaration),
    /// Usually a local variable, introduced in your telescope
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
    ///
    /// Members: pattern, parameter type (optional), body expression and the captured scope.
    Abstraction(Pattern, Option<Box<Value>>, Expression, Box<Telescope>),
    /// This is not present in Mini-TT.<br/>
    /// Sometimes the closure is already an evaluated value.
    Value(Box<Value>),
    /// `clCmp` in Mini-TT.<br/>
    /// Closure that was inside of a case-split.
    ///
    /// For example, in a definition:
    /// ```ignore
    /// f = split { TT a => bla };
    /// ```
    /// The part `TT a => bla` is a choice closure, where `Box<Self>` refers to the `a => bla` part
    /// and `TT` is the `String`.
    Choice(Box<Self>, String),
}

/// Generic definition for two kinds of case trees
pub type GenericCaseTree<Value> = (Box<Branch>, Box<Rc<GenericTelescope<Value>>>);

/// `SClos` in Mini-TT.<br/>
/// Case tree.
pub type CaseTree = GenericCaseTree<Value>;
