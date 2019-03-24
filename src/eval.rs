use std::cmp::max;

use crate::ast::*;

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
    pub fn project(&self, name: &str, val: Value) -> Result<Value, String> {
        match self {
            Pattern::Pair(first, second) => {
                if first.contains(name) {
                    first.project(name, val.first())
                } else if second.contains(name) {
                    second.project(name, val.second())
                } else {
                    Err(format!("Cannot project with `{}`", name))
                }
            }
            Pattern::Var(pattern_name) => {
                if pattern_name == name {
                    Ok(val)
                } else {
                    Err(format!(
                        "Expected projection: `{}`, found: `{}`.",
                        pattern_name, name
                    ))
                }
            }
            Pattern::Unit => Err("Cannot project unit pattern".to_string()),
        }
    }
}

impl TelescopeRaw {
    /// `getRho` in Mini-TT.
    pub fn resolve(&self, name: &str) -> Result<Value, String> {
        use crate::ast::GenericTelescope::*;
        match self {
            Nil => Err(format!("Unresolved reference: `{}`.", name)),
            UpDec(context, declaration) => {
                let pattern = &declaration.pattern;
                if pattern.contains(name) {
                    pattern.project(
                        name,
                        declaration.body.clone().eval(if declaration.is_recursive {
                            up_dec_rc(context.clone(), declaration.clone())
                        } else {
                            context.clone()
                        }),
                    )
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
        match self {
            Closure::Abstraction(pattern, _, expression, context) => {
                expression.eval(up_var_rc(*context, pattern, value))
            }
            Closure::Value(value) => *value,
            Closure::Choice(closure, name) => {
                closure.instantiate(Value::Constructor(name, Box::new(value)))
            }
        }
    }
}

impl Value {
    /// Calculate the level of `self`, return `None` if it's not a type value.
    pub fn level_safe(&self) -> Option<Level> {
        match self {
            Value::One => Some(0),
            Value::Type(level) => Some(1 + level),
            Value::Pi(_, _, level) | Value::Sigma(_, _, level) | Value::Sum(_, level) => {
                Some(*level)
            }
            _ => None,
        }
    }

    /// This is called `levelView` in Agda.
    pub fn level(&self) -> u32 {
        self.level_safe()
            .unwrap_or_else(|| panic!("Cannot calculate the level of: `{}`.", self))
    }

    /// `vfst` in Mini-TT.<br/>
    /// Run `.1` on a Pair.
    pub fn first(self) -> Self {
        use crate::ast::GenericNeutral as Neutral;
        match self {
            Value::Pair(first, _) => *first,
            Value::Neutral(neutral) => Value::Neutral(Neutral::First(Box::new(neutral))),
            e => panic!("Cannot first: `{}`.", e),
        }
    }

    /// `vsnd` in Mini-TT.<br/>
    /// Run `.2` on a Pair.
    pub fn second(self) -> Self {
        use crate::ast::GenericNeutral as Neutral;
        match self {
            Value::Pair(_, second) => *second,
            Value::Neutral(neutral) => Value::Neutral(Neutral::Second(Box::new(neutral))),
            e => panic!("Cannot second: `{}`.", e),
        }
    }

    /// Combination of `vsnd` and `vfst` in Mini-TT.<br/>
    /// Run `.2` on a Pair.
    pub fn destruct(self) -> (Self, Self) {
        use crate::ast::GenericNeutral as Neutral;
        match self {
            Value::Pair(first, second) => (*first, *second),
            Value::Neutral(neutral) => (
                Value::Neutral(Neutral::First(Box::new(neutral.clone()))),
                Value::Neutral(Neutral::Second(Box::new(neutral))),
            ),
            e => panic!("Cannot destruct: `{}`.", e),
        }
    }

    /// `app` in Mini-TT.
    pub fn apply(self, argument: Self) -> Self {
        use crate::ast::GenericNeutral as Neutral;
        match self {
            Value::Lambda(closure) => closure.instantiate(argument),
            Value::Split(case_tree) => match argument {
                Value::Constructor(name, body) => case_tree
                    .get(&name)
                    .unwrap_or_else(|| panic!("Cannot find constructor `{}`.", name))
                    .clone()
                    .reduce_to_value()
                    .apply(*body),
                Value::Neutral(neutral) => {
                    Value::Neutral(Neutral::Split(case_tree, Box::new(neutral)))
                }
                e => panic!("Cannot apply a: `{}`.", e),
            },
            Value::Neutral(neutral) => {
                Value::Neutral(Neutral::Application(Box::new(neutral), Box::new(argument)))
            }
            e => panic!("Cannot apply on: `{}`.", e),
        }
    }
}

impl Expression {
    /// `eval` in Mini-TT.<br/>
    /// Evaluate an [`Expression`] to a [`Value`] under a [`Telescope`],
    /// panic if not well-typed.
    pub fn eval(self, context: Telescope) -> Value {
        use crate::ast::Expression as E;
        use crate::ast::Value as V;
        match self {
            E::Unit => V::Unit,
            E::One => V::One,
            E::Type(level) => V::Type(level),
            E::Var(name) => context
                .resolve(&name)
                .map_err(|err| eprintln!("{}", err))
                .unwrap(),
            // FIXME: this will cause infinite loop when type-checking recursive sum
            E::Sum(constructors, level) => {
                let level = level.unwrap_or_else(|| {
                    let mut max_level: Level = 0;
                    // for (_name, case) in constructors.iter() {
                    //     let level = case.clone().eval(context.clone()).level();
                    //     max_level = max(max_level, level);
                    // }
                    max_level
                });
                V::Sum(branch_to_righted(constructors, context), level)
            }
            E::Merge(left, right) => {
                let (mut left, left_level) = match left.eval(context.clone()) {
                    V::Sum(constructors, level) => (constructors, level),
                    otherwise => panic!("Not a Sum expression: `{}`.", otherwise),
                };
                let (mut right, right_level) = match right.eval(context) {
                    V::Sum(constructors, level) => (constructors, level),
                    otherwise => panic!("Not a Sum expression: `{}`.", otherwise),
                };
                left.append(&mut right);
                V::Sum(left, max(left_level, right_level))
            }
            E::Split(case_tree) => V::Split(branch_to_righted(case_tree, context)),
            E::Pi(input, output, level) => {
                let pattern = input.pattern;
                let input = Box::new(input.expression.eval(context.clone()));
                let extra_info = Some(input.clone());
                let output = Closure::Abstraction(pattern, extra_info, *output, Box::new(context));
                let level = level.unwrap_or_else(|| {
                    // let output_level = output.clone().instantiate(generate_value(0)).level();
                    // max(input.level(), output_level)
                    0
                });
                V::Pi(input, output, level)
            }
            E::Sigma(first, second, level) => {
                let pattern = first.pattern;
                let first = Box::new(first.expression.eval(context.clone()));
                let extra_info = Some(first.clone());
                let second = Closure::Abstraction(pattern, extra_info, *second, Box::new(context));
                let level = level.unwrap_or_else(|| {
                    // let second_level = second.clone().instantiate(generate_value(0)).level();
                    // max(first.level(), second_level)
                    0
                });
                V::Sigma(first, second, level)
            }
            E::Lambda(pattern, parameter_type, body) => V::Lambda(Closure::Abstraction(
                pattern,
                parameter_type.map(|t| t.internal),
                *body,
                Box::new(context),
            )),
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
            E::Constant(pattern, expression, rest) => rest.eval(up_var_rc(
                context.clone(),
                pattern,
                expression.eval(context),
            )),
            e => panic!("Cannot eval: {}", e),
        }
    }
}
