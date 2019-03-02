use std::rc::Rc;

use crate::syntax::*;

impl Pattern {
    /// `inPat` in Mini-TT.
    pub fn contains(&self, name: &str) -> bool {
        match self {
            Pattern::Var(pattern_name) => pattern_name == name,
            Pattern::Pair(first, second) => first.contains(name) || second.contains(name),
            Pattern::Unit => false,
        }
    }

    /// `patProj` in Mini-TT.
    pub fn project(&self, name: &str, val: Value) -> Value {
        match self {
            Pattern::Pair(first, second) => {
                if first.contains(name) {
                    first.project(name, val.first())
                } else if second.contains(name) {
                    second.project(name, val.second())
                } else {
                    panic!("Cannot project with `{}`", name)
                }
            }
            Pattern::Var(pattern_name) => {
                if pattern_name == name {
                    val
                } else {
                    panic!("Expected projection: `{}`, found: `{}`", pattern_name, name)
                }
            }
            Pattern::Unit => panic!("Cannot project unit pattern"),
        }
    }
}

impl TelescopeRaw {
    /// `getRho` in Mini-TT.
    pub fn resolve(&self, name: &str) -> Value {
        use crate::syntax::GenericTelescope::*;
        match self {
            Nil => panic!("Unresolved reference: `{}`", name),
            UpDec(context, Declaration::Simple(pattern, _, expression))
            | UpDec(context, Declaration::Recursive(pattern, _, expression)) => {
                if pattern.contains(name) {
                    pattern.project(name, expression.clone().eval(context.clone()))
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

impl Closure {
    /// `*` in Mini-TT.<br/>
    /// Instantiate a closure with `val`.
    pub fn instantiate(self, value: Value) -> Value {
        use crate::syntax::GenericTelescope as Telescope;
        match self {
            Closure::Abstraction(pattern, expression, context) => {
                expression.eval(Rc::new(Telescope::UpVar(*context, pattern, value)))
            }
            Closure::Value(value) => *value,
            Closure::Choice(closure, name) => {
                closure.instantiate(Value::Constructor(name, Box::new(value)))
            }
        }
    }
}

impl Value {
    /// `vfst` in Mini-TT.<br/>
    /// Run `.1` on a Pair.
    pub fn first(self) -> Value {
        use crate::syntax::GenericNeutral as Neutral;
        match self {
            Value::Pair(first, _) => *first,
            Value::Neutral(neutral) => Value::Neutral(Neutral::First(Box::new(neutral))),
            e => panic!("Cannot first: {}", e),
        }
    }

    /// `vsnd` in Mini-TT.<br/>
    /// Run `.2` on a Pair.
    pub fn second(self) -> Value {
        use crate::syntax::GenericNeutral as Neutral;
        match self {
            Value::Pair(_, second) => *second,
            Value::Neutral(neutral) => Value::Neutral(Neutral::Second(Box::new(neutral))),
            e => panic!("Cannot second: {}", e),
        }
    }

    /// Combination of `vsnd` and `vfst` in Mini-TT.<br/>
    /// Run `.2` on a Pair.
    pub fn destruct(self) -> (Value, Value) {
        use crate::syntax::GenericNeutral as Neutral;
        match self {
            Value::Pair(first, second) => (*first, *second),
            Value::Neutral(neutral) => (
                Value::Neutral(Neutral::First(Box::new(neutral.clone()))),
                Value::Neutral(Neutral::Second(Box::new(neutral))),
            ),
            e => panic!("Cannot destruct: {}", e),
        }
    }

    /// `app` in Mini-TT.
    pub fn apply(self, argument: Value) -> Value {
        use crate::syntax::GenericNeutral as Neutral;
        match self {
            Value::Lambda(closure) => closure.instantiate(argument),
            Value::Split((case_tree, context)) => match argument {
                Value::Constructor(name, body) => case_tree
                    .get(&name)
                    .unwrap_or_else(|| panic!("Cannot find constructor `{}`.", name))
                    .clone()
                    .eval(*context)
                    .apply(*body),
                Value::Neutral(neutral) => {
                    Value::Neutral(Neutral::Split((case_tree, context), Box::new(neutral)))
                }
                e => panic!("Cannot apply a: {}", e),
            },
            Value::Neutral(neutral) => {
                Value::Neutral(Neutral::Application(Box::new(neutral), Box::new(argument)))
            }
            e => panic!("Cannot apply on: {}", e),
        }
    }
}

impl Expression {
    /// `eval` in Mini-TT.<br/>
    /// Evaluate an [`Expression`] to a [`Value`] under a [`Telescope`],
    /// panic if not well-typed.
    pub fn eval(self, context: Telescope) -> Value {
        use crate::syntax::Expression as E;
        use crate::syntax::Value as V;
        match self {
            E::Unit => V::Unit,
            E::One => V::One,
            E::Type => V::Type,
            E::Var(name) => context
                .resolve(&name),
            E::Sum(constructors) => V::Sum((Box::new(constructors), Box::new(context))),
            E::Split(case_tree) => V::Split((Box::new(case_tree), Box::new(context))),
            E::Pi(pattern, first, second) => V::Pi(
                Box::new(first.eval(context.clone())),
                Closure::Abstraction(pattern, *second, Box::new(context)),
            ),
            E::Sigma(pattern, first, second) => V::Sigma(
                Box::new(first.eval(context.clone())),
                Closure::Abstraction(pattern, *second, Box::new(context)),
            ),
            E::Lambda(pattern, body) => {
                V::Lambda(Closure::Abstraction(pattern, *body, Box::new(context)))
            }
            E::First(pair) => pair.eval(context).first(),
            E::Second(pair) => pair.eval(context).second(),
            E::Application(function, argument) => {
                function.eval(context.clone()).apply(argument.eval(context))
            }
            E::Pair(first, second) => V::Pair(
                Box::new(first.eval(context.clone())),
                Box::new(second.eval(context)),
            ),
            E::Constructor(name, body) => V::Constructor(name, Box::new(body.eval(context))),
            E::Declaration(declaration, rest) => rest.eval(up_dec_rc(context, *declaration)),
            e => panic!("Cannot eval: {}", e),
        }
    }
}
