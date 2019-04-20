use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;

use either::Either;

use crate::ast::*;

/// `NRho` in Mini-TT, normal form telescopes (contexts).
pub type NormalTelescope = Rc<GenericTelescope<NormalExpression>>;

pub type NormalCase = GenericCase<Either<NormalExpression, Expression>, NormalExpression>;

/// `NSClos` in Mini-TT, normal form closures.
pub type NormalCaseTree = GenericBranch<NormalCase>;

/// `NNeut` in Mini-TT, normal form neutral values.
pub type NormalNeutral = GenericNeutral<NormalExpression>;

/// `NExp` in Mini-TT, normal form expressions.<br/>
/// Deriving `Eq` so we can do comparison.
///
/// $E ::=$
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NormalExpression {
    /// $\lambda \textsf{x}_i\  .\ E$
    Lambda(u32, Box<Self>),
    /// $E_1, E_2$
    Pair(Box<Self>, Box<Self>),
    /// $0$
    Unit,
    /// $\textbf{1}$
    One,
    /// $\textsf{U}$
    Type(Level),
    /// $\Pi \textsf{x}_i:E_1.E_2$
    Pi(Box<Self>, u32, Box<Self>),
    /// $\Sigma \textsf{x}_i:E_1.E_2$
    Sigma(Box<Self>, u32, Box<Self>),
    /// $c\ E$
    Constructor(String, Box<Self>),
    /// $\textsf{fun}\lang S,\alpha \rang$
    Split(NormalCaseTree),
    /// $\textsf{Sum}\lang S,\alpha \rang$
    Sum(NormalCaseTree),
    /// $[K]$
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

    /// $$
    /// \begin{alignedat}{2}
    ///   & \textsf{R}_i (\lambda \ f) &&= \lambda \ \textsf{x}_i
    ///      \ . \ \textsf{R}\_{i+1} (\textsf{inst}\ f [ \textsf{x}\_i ]) \\\\
    ///   & \textsf{R}_i (u,v) &&= (\textsf{R}_i u,\textsf{R}_i v) \\\\
    ///   & \textsf{R}_i 0 &&= 0 \\\\
    ///   & \textsf{R}_i (c\ v) &&= c \ (\textsf{R}_i v) \\\\
    ///   & \textsf{R}_i (\textsf{fun}\lang S,\rho\rang) &&= \textsf{fun}\lang S,
    ///      \textsf{R}_i \rho\rang \\\\
    ///   & \textsf{R}_i (\textsf{Sum}\lang S,\rho\rang) &&= \textsf{Sum}\lang S,
    ///      \textsf{R}_i \rho\rang \\\\
    ///   & \textsf{R}_i \textsf{U} &&= \textsf{U} \\\\
    ///   & \textsf{R}_i \textbf{1} &&= \textbf{1} \\\\
    ///   & \textsf{R}_i (\Pi\ t\ g) &&= \Pi \textsf{x}_i : \textsf{R}\_i \ t
    ///      \ . \ \textsf{R}\_{i+1} (\textsf{inst}\ g [ \textsf{x}\_i ]) \\\\
    ///   & \textsf{R}_i (\Sigma\ t\ g) &&= \Sigma \textsf{x}_i : \textsf{R}\_i \ t
    ///      \ . \ \textsf{R}\_{i+1} (\textsf{inst}\ g [ \textsf{x}\_i ]) \\\\
    ///   & \textsf{R}_i [k] &&= [\textsf{R}\_i\ k] \\\\
    ///   & && \\\\
    ///   & \textsf{R}_i \textsf{x}\_j &&= \textsf{x}\_j \\\\
    ///   & \textsf{R}_i (k\ v) &&= (\textsf{R}_i\ k) (\textsf{R}_i\ v) \\\\
    ///   & \textsf{R}_i (k.1) &&= (\textsf{R}_i\ k) .1 \\\\
    ///   & \textsf{R}_i (k.2) &&= (\textsf{R}_i\ k) .2 \\\\
    ///   & \textsf{R}_i (\lang S,\rho \rang\ k) &&=
    ///      \lang S,\textsf{R}\_i \rho \rang (\textsf{R}_i\ k) \\\\
    ///   & && \\\\
    ///   & \textsf{R}_i (\rho,p=v) &&= \textsf{R}_i\ \rho,p=\textsf{R}\_i\ v \\\\
    ///   & \textsf{R}_i (\rho,D) &&= \textsf{R}_i\ \rho,D \\\\
    ///   & \textsf{R}_i () &&= ()
    /// \end{alignedat}
    /// $$
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

    /// $$
    /// \begin{alignedat}{2}
    ///   & \textsf{R}_i (\lambda \ f) &&= \lambda \ \textsf{x}_i
    ///      \ . \ \textsf{R}\_{i+1} (\textsf{inst}\ f [ \textsf{x}\_i ]) \\\\
    ///   & \textsf{R}_i (u,v) &&= (\textsf{R}_i u,\textsf{R}_i v) \\\\
    ///   & \textsf{R}_i 0 &&= 0 \\\\
    ///   & \textsf{R}_i (c\ v) &&= c \ (\textsf{R}_i v) \\\\
    ///   & \textsf{R}_i (\textsf{fun}\lang S,\rho\rang) &&= \textsf{fun}\lang S,
    ///      \textsf{R}_i \rho\rang \\\\
    ///   & \textsf{R}_i (\textsf{Sum}\lang S,\rho\rang) &&= \textsf{Sum}\lang S,
    ///      \textsf{R}_i \rho\rang \\\\
    ///   & \textsf{R}_i \textsf{U} &&= \textsf{U} \\\\
    ///   & \textsf{R}_i \textbf{1} &&= \textbf{1} \\\\
    ///   & \textsf{R}_i (\Pi\ t\ g) &&= \Pi \textsf{x}_i : \textsf{R}\_i \ t
    ///      \ . \ \textsf{R}\_{i+1} (\textsf{inst}\ g [ \textsf{x}\_i ]) \\\\
    ///   & \textsf{R}_i (\Sigma\ t\ g) &&= \Sigma \textsf{x}_i : \textsf{R}\_i \ t
    ///      \ . \ \textsf{R}\_{i+1} (\textsf{inst}\ g [ \textsf{x}\_i ]) \\\\
    ///   & \textsf{R}_i [k] &&= [\textsf{R}\_i\ k]
    /// \end{alignedat}
    /// $$
    /// `rbV` in Mini-TT.
    fn read_back(self, index: u32) -> Self::NormalForm {
        use crate::check::read_back::NormalExpression::*;
        match self {
            Value::Lambda(closure) => {
                let closure = closure
                    .instantiate(generate_value(index))
                    .read_back(index + 1);
                Lambda(index, Box::new(closure))
            }
            Value::Unit => Unit,
            Value::One => One,
            Value::Type(level) => Type(level),
            Value::Pi(input, output) => {
                let output = output
                    .instantiate(generate_value(index))
                    .read_back(index + 1);
                Pi(Box::new(input.read_back(index)), index, Box::new(output))
            }
            Value::Sigma(first, second) => {
                let second = second
                    .instantiate(generate_value(index))
                    .read_back(index + 1);
                Sigma(Box::new(first.read_back(index)), index, Box::new(second))
            }
            Value::Pair(first, second) => Pair(
                Box::new(first.read_back(index)),
                Box::new(second.read_back(index)),
            ),
            Value::Constructor(name, body) => Constructor(name, Box::new(body.read_back(index))),
            Value::Split(case_tree) => Split(read_back_branches(index, case_tree)),
            Value::Sum(constructors) => Sum(read_back_branches(index, constructors)),
            Value::Neutral(neutral) => Neutral(neutral.read_back(index)),
        }
    }
}

fn read_back_branches(index: u32, branches: CaseTree) -> NormalCaseTree {
    let mut read_back_constructors = BTreeMap::new();
    for (name, case) in branches.into_iter() {
        read_back_constructors.insert(name, Box::new(case.read_back(index)));
    }
    read_back_constructors
}

impl ReadBack for Case {
    type NormalForm = NormalCase;

    fn read_back(self, index: u32) -> Self::NormalForm {
        Self::NormalForm::new(
            self.expression.map_left(|l| l.read_back(index)),
            self.context.read_back(index),
        )
    }
}

impl ReadBack for &GenericTelescope<Value> {
    type NormalForm = NormalTelescope;

    /// $$
    /// \begin{alignedat}{2}
    ///   & \textsf{R}_i (\rho,p=v) &&= \textsf{R}_i\ \rho,p=\textsf{R}\_i\ v \\\\
    ///   & \textsf{R}_i (\rho,D) &&= \textsf{R}_i\ \rho,D \\\\
    ///   & \textsf{R}_i () &&= ()
    /// \end{alignedat}
    /// $$
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

    /// $$
    /// \begin{alignedat}{2}
    ///   & \textsf{R}_i \textsf{x}\_j &&= \textsf{x}\_j \\\\
    ///   & \textsf{R}_i (k\ v) &&= (\textsf{R}_i\ k) (\textsf{R}_i\ v) \\\\
    ///   & \textsf{R}_i (k.1) &&= (\textsf{R}_i\ k) .1 \\\\
    ///   & \textsf{R}_i (k.2) &&= (\textsf{R}_i\ k) .2 \\\\
    ///   & \textsf{R}_i (\lang S,\rho \rang\ k) &&=
    ///     \lang S,\textsf{R}\_i \rho \rang (\textsf{R}_i\ k)
    /// \end{alignedat}
    /// $$
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
            Split(case_tree, body) => Split(
                read_back_branches(index, case_tree),
                Box::new(body.read_back(index)),
            ),
        }
    }
}
