use crate::syntax::*;
use std::fmt::Debug;

pub trait DebuggableNameTrait: NameTrait + Debug {}

impl<Name: DebuggableNameTrait> Pattern<Name> {
    /// `inPat` in Mini-TT.
    pub fn contains(&self, name: &Name) -> bool {
        match self {
            Pattern::Var(pattern_name) => pattern_name == name,
            Pattern::Pair(first, second) => first.contains(name) || second.contains(name),
            Pattern::Unit => false,
        }
    }

    /// `patProj` in Mini-TT.
    pub fn project(&self, name: &Name, val: Value<Name>) -> Value<Name> {
        match self {
            Pattern::Pair(first, second) => {
                if first.contains(name) {
                    first.project(name, val.first())
                } else if second.contains(name) {
                    second.project(name, val.second())
                } else {
                    panic!("Cannot project with {:?}", name)
                }
            }
            Pattern::Var(pattern_name) => {
                if pattern_name == name {
                    val
                } else {
                    panic!("Expected projection: {:?}, found: {:?}", pattern_name, name)
                }
            }
            Pattern::Unit => panic!("Cannot project unit pattern"),
        }
    }
}

impl<Name: DebuggableNameTrait> Telescope<Name> {
    /// `getRho` in Mini-TT.
    pub fn resolve(&self, name: &Name) -> Value<Name> {
        use crate::syntax::GenericTelescope::*;
        match self {
            Nil => panic!("Unresolved reference: {:?}", name),
            UpDec(context, Declaration::Simple(pattern, _, expression))
            | UpDec(context, Declaration::Recursive(pattern, _, expression)) => {
                if pattern.contains(name) {
                    pattern.project(name, expression.clone().eval(context))
                } else {
                    context.resolve(name)
                }
            }
            UpVar(context, pattern, val) => {
                if pattern.contains(name) {
                    pattern.project(name, val.clone())
                } else {
                    context.resolve(name)
                }
            }
        }
    }
}

impl<Name: DebuggableNameTrait> Closure<Name> {
    /// `*` in Mini-TT.
    /// Instantiate a closure.
    pub fn instantiate(self, val: Value<Name>) -> Value<Name> {
        use crate::syntax::GenericTelescope as Telescope;
        match self {
            Closure::Choice(pattern, expression, context) => {
                expression.eval(&Telescope::UpVar(context, pattern, val))
            }
            Closure::Function(closure, name) => {
                closure.instantiate(Value::Constructor(name, Box::new(val)))
            }
        }
    }
}

impl<Name: DebuggableNameTrait> Value<Name> {
    /// `vfst` in Mini-TT. Run `.1` on a Pair.
    pub fn first(self) -> Value<Name> {
        use crate::syntax::GenericNeutral as Neutral;
        match self {
            Value::Pair(first, _) => *first,
            Value::Neutral(neutral) => Value::Neutral(Neutral::First(Box::new(neutral))),
            e => panic!(format!("Cannot first: {:?}", e)),
        }
    }

    /// `vsnd` in Mini-TT. Run `.2` on a Pair.
    pub fn second(self) -> Value<Name> {
        use crate::syntax::GenericNeutral as Neutral;
        match self {
            Value::Pair(_, second) => *second,
            Value::Neutral(neutral) => Value::Neutral(Neutral::Second(Box::new(neutral))),
            e => panic!("Cannot second: {:?}", e),
        }
    }

    /// `app` in Mini-TT.
    pub fn apply(self, argument: Value<Name>) -> Value<Name> {
        use crate::syntax::GenericNeutral as Neutral;
        match self {
            Value::Lambda(closure) => closure.instantiate(argument),
            Value::Function((case_tree, context)) => match argument {
                Value::Constructor(name, body) => case_tree
                    .get(&name)
                    .unwrap_or_else(|| panic!("Cannot find constructor {:?}.", name))
                    .clone()
                    .eval(&context)
                    .apply(*body),
                Value::Neutral(neutral) => {
                    Value::Neutral(Neutral::Function((case_tree, context), Box::new(neutral)))
                }
                e => panic!("Cannot apply a: {:?}", e),
            },
            Value::Neutral(neutral) => {
                Value::Neutral(Neutral::Application(Box::new(neutral), Box::new(argument)))
            }
            e => panic!("Cannot apply on: {:?}", e),
        }
    }
}

impl<Name: DebuggableNameTrait> Expression<Name> {
    /// `eval` in Mini-TT.
    /// Evaluate an [`Expression`] to a [`Value`] under a [`Telescope`].
    /// Will panic if not well-typed.
    pub fn eval(self, context: &Telescope<Name>) -> Value<Name> {
        use crate::syntax::GenericTelescope as Telescope;
        match self {
            Expression::Unit => Value::Unit,
            Expression::One => Value::One,
            Expression::Type => Value::Type,
            Expression::Var(name) => context.resolve(&name),
            Expression::Sum(constructors) => {
                Value::Sum((Box::new(constructors), Box::new(context.clone())))
            }
            Expression::Function(case_tree) => {
                Value::Function((Box::new(case_tree), Box::new(context.clone())))
            }
            Expression::Pi(pattern, first, second) => Value::Pi(
                Box::new(first.eval(context)),
                Closure::Choice(pattern, *second, Box::new(context.clone())),
            ),
            Expression::Sigma(pattern, first, second) => Value::Sigma(
                Box::new(first.eval(context)),
                Closure::Choice(pattern, *second, Box::new(context.clone())),
            ),
            Expression::Lambda(pattern, body) => {
                Value::Lambda(Closure::Choice(pattern, *body, Box::new(context.clone())))
            }
            Expression::First(pair) => pair.eval(context).first(),
            Expression::Second(pair) => pair.eval(context).second(),
            Expression::Application(function, argument) => {
                function.eval(context).apply(argument.eval(context))
            }
            Expression::Pair(first, second) => Value::Pair(
                Box::new(first.eval(context)),
                Box::new(second.eval(context)),
            ),
            Expression::Constructor(name, body) => {
                Value::Constructor(name, Box::new(body.eval(context)))
            }
            Expression::Declaration(declaration, rest) => {
                rest.eval(&Telescope::UpDec(Box::new(context.clone()), *declaration))
            }
            e => panic!("Cannot eval: {:?}", e),
        }
    }
}
