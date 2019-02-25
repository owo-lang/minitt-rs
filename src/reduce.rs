use crate::syntax::Closure;
use crate::syntax::Expression;
use crate::syntax::NameTrait;
use crate::syntax::Neutral;
use crate::syntax::Telescope;
use crate::syntax::Telescope::UpDec;
use crate::syntax::Value;
use std::fmt::Debug;

pub trait NameTraitAndDebug: NameTrait + Debug {}

impl<Name: NameTraitAndDebug> Closure<Name> {
    /// `*` in Mini-TT.
    /// Instantiate a closure.
    pub fn instantiate(self, val: Value<Name>) -> Value<Name> {
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

impl<Name: NameTraitAndDebug> Value<Name> {
    /// `vfst` in Mini-TT. Run `.1` on a Pair.
    pub fn value_first(self) -> Value<Name> {
        match self {
            Value::Pair(first, _) => *first,
            Value::Neutral(neutral) => Value::Neutral(Neutral::First(Box::new(neutral))),
            e => panic!(format!("Cannot first: {:?}", e)),
        }
    }

    /// `vsnd` in Mini-TT. Run `.2` on a Pair.
    pub fn value_second(self) -> Value<Name> {
        match self {
            Value::Pair(_, second) => *second,
            Value::Neutral(neutral) => Value::Neutral(Neutral::Second(Box::new(neutral))),
            e => panic!(format!("Cannot second: {:?}", e)),
        }
    }

    /// `app` in Mini-TT.
    pub fn apply(self, argument: Value<Name>) -> Value<Name> {
        match self {
            Value::Lambda(closure) => closure.instantiate(argument),
            Value::Function((case_tree, context)) => match argument {
                Value::Constructor(name, body) => case_tree
                    .get(&name)
                    .expect(format!("Cannot find constructor {:?}.", name).as_str())
                    .clone()
                    .eval(&context)
                    .apply(*body),
                Value::Neutral(neutral) => Value::Neutral(Neutral::Function(
                    Box::new((case_tree, context)),
                    Box::new(neutral),
                )),
                e => panic!(format!("Cannot apply a: {:?}", e)),
            },
            Value::Neutral(neutral) => {
                Value::Neutral(Neutral::Application(Box::new(neutral), Box::new(argument)))
            }
            e => panic!(format!("Cannot apply on: {:?}", e)),
        }
    }
}

impl<Name: NameTraitAndDebug> Expression<Name> {
    /// `eval` in Mini-TT.
    /// Evaluate an [`Expression`] to a [`Value`] under a [`Telescope`].
    /// Will panic if not well-typed.
    pub fn eval(self, context: &Telescope<Name>) -> Value<Name> {
        unimplemented!()
    }
}
