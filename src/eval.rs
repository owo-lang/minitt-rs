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

    /// $$
    /// \begin{alignedat}{2}
    ///  & \textsf{proj}^x_x(v) &&= v \\\\
    ///  & \textsf{proj}^{(p_1,p_2)}_x (v) &&= \textsf{proj}^{p_1}_x (v.1)
    ///   \ \textnormal{if}\ x\ \textnormal{is\ in}\ p_1, \\\\
    ///  & \textsf{proj}^{(p_1,p_2)}_x (v) &&= \textsf{proj}^{p_2}_x (v.2)
    ///   \ \textnormal{if}\ x\ \textnormal{is\ in}\ p_2
    /// \end{alignedat}
    /// $$
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

impl GenericTelescope<Value> {
    /// $$
    /// \textnormal{If} \ x \ \textnormal{is\ in}\ p, \\\\
    /// \begin{alignedat}{2}
    ///   & (\rho, p=v)(x) &&= \textsf{proj}^p_x(v) \\\\
    ///   & (\rho, p:A=M)(x) &&= \textsf{proj}^p_x(⟦ M ⟧ \rho) \\\\
    ///   & (\rho, \textsf{rec}\ p:A=M)(x) &&=
    ///     \textsf{proj}^p_x(⟦ M ⟧ (\rho, \textsf{rec}\ p:A=M))
    /// \end{alignedat} \\\\
    /// \textnormal{If} \ x \ \textnormal{is\ not\ in}\ p, \\\\
    /// \begin{alignedat}{2}
    ///   & (\rho, p=v)(x) &&= \rho(x) \\\\
    ///   & (\rho, D)(x) &&= \rho(x) \\\\
    /// \end{alignedat}
    /// $$
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
    /// $$
    /// \begin{alignedat}{2}
    ///   & \textsf{inst} \lang \lambda p.M, \rho \rang v &&= ⟦ M ⟧ (\rho,p=v) \\\\
    ///   & \textsf{inst} (f \circ c) v &&= \textsf{inst} \ f (c \ v)
    /// \end{alignedat}
    /// $$
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
    /// This is not present in Mini-TT.<br/>
    /// Calculate the level of `self`, return `None` if it's not a type value.
    pub fn level_safe(&self) -> Option<Level> {
        use crate::ast::Value::*;
        match self {
            One => Some(0),
            Type(level) => Some(1 + level),
            Sum(branches) => Some(
                branches
                    .iter()
                    .map(|(_, case)| case.clone().reduce_to_value().level_safe().unwrap_or(0))
                    .max()
                    .map(|l| if l > 0 { l - 1 } else { 0 })
                    .unwrap_or(0),
            ),
            Pi(first, second) | Value::Sigma(first, second) => Some(max(
                first
                    .level_safe()
                    .map_or(0, |level| if level > 1 { level - 1 } else { 0 }),
                // todo: same problem as line 100, and
                // will `.instantiate(Value::Unit)` work well when calculate level?
                second
                    .clone()
                    .instantiate(Value::Unit)
                    .level_safe()
                    .map_or(0, |level| if level > 1 { level - 1 } else { 0 }),
            )),
            _ => None,
        }
    }

    /// This is not present in Mini-TT.<br/>
    /// This is called `levelView` in Agda.
    pub fn level(&self) -> u32 {
        self.level_safe()
            .unwrap_or_else(|| panic!("Cannot calculate the level of: `{}`.", self))
    }

    /// $$
    /// \begin{alignedat}{2}
    ///   & (u,v).1 &&= u \\\\
    ///   & [k].1 &&= [k.1] \\\\
    /// \end{alignedat}
    /// $$
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

    /// $$
    /// \begin{alignedat}{2}
    ///   & (u,v).2 &&= v \\\\
    ///   & [k].2 &&= [k.2] \\\\
    /// \end{alignedat}
    /// $$
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

    /// $$
    /// \begin{alignedat}{2}
    ///  & \textsf{app} (\lambda \ f) v &&= \textsf{inst} \ f \ v \\\\
    ///  & \textsf{app} (\textsf{fun}\lang S,\rho \rang (c_i \ v)) &&=
    ///       \textsf{app}(⟦ M\_i ⟧ \rho)v \\\\
    ///  &    && \ \ \ \ \ \ \textnormal{where}
    ///       \ S=(c_1 \rightarrow M_1 | ... | c_n \rightarrow M_n) \\\\
    ///  & \textsf{app} (\textsf{fun} \ s) [k] &&= [s \ k] \\\\
    ///  & \textsf{app} [k] \ v &&= [k \ v]
    /// \end{alignedat}
    /// $$
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
    /// This is not present in Mini-TT.<br/>
    /// Return `true` if `self` is a `Sum` or `Merge`.
    /// This is quite expensive! Can we optimize it a little bit?
    pub fn eval_to_sum(self, context: Telescope) -> Option<Vec<String>> {
        match self.eval(context) {
            Value::Sum(constructors) => Some(constructors.keys().cloned().collect()),
            _ => None,
        }
    }

    /// $$
    /// \begin{alignedat}{2}
    ///   & ⟦ \lambda p.M ⟧ \rho &&= \lang \lambda p.M,\rho \rang \\\\
    ///   & ⟦ x ⟧ \rho &&= \rho(x) \\\\
    ///   & ⟦ M \ N ⟧ \rho &&= \textsf{app} (⟦ M ⟧ \rho, ⟦ N ⟧ \rho) \\\\
    ///   & ⟦ \Pi \ p:A.B ⟧ \rho &&= \Pi (⟦ A ⟧ \rho) \lang \lambda p.B,\rho \rang \\\\
    ///   & ⟦ \textsf{U} ⟧ \rho &&= \textsf{U} \\\\
    ///   & ⟦ D; M ⟧ \rho &&= ⟦ M ⟧ (\rho(x); D) \\\\
    ///  & && \\\\
    ///   & ⟦ M,N ⟧ \rho &&= (⟦ M ⟧ \rho, ⟦ N ⟧ \rho) \\\\
    ///   & ⟦ 0 ⟧ \rho &&= 0 \\\\
    ///   & ⟦ M.1 ⟧ \rho &&= (⟦ M\rho ⟧).1 \\\\
    ///   & ⟦ M.2 ⟧ \rho &&= (⟦ M\rho ⟧).2 \\\\
    ///   & ⟦ \Sigma \ p:A.B ⟧ \rho &&= \Sigma (⟦ A ⟧ \rho) \lang \lambda p.B,\rho \rang \\\\
    ///   & ⟦ \textbf{1} ⟧ \rho &&= \textbf{1} \\\\
    ///  & && \\\\
    ///   & ⟦ c \ M ⟧ \rho &&= c(⟦ M ⟧ \rho) \\\\
    ///   & ⟦ \textsf{fun} \ S ⟧ \rho &&= \textsf{fun} \lang S,\rho \rang \\\\
    ///   & ⟦ \textsf{Sum} \ S ⟧ \rho &&= \textsf{Sum} \lang S,\rho \rang
    /// \end{alignedat}
    /// $$
    /// `eval` in Mini-TT.<br/>
    /// Evaluate an `Expression` to a `Value` under a `Telescope`,
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
            E::Sum(constructors) => V::Sum(branch_to_righted(constructors, context)),
            E::Merge(left, right) => {
                let mut left = match left.eval(context.clone()) {
                    V::Sum(constructors) => constructors,
                    otherwise => panic!("Not a Sum expression: `{}`.", otherwise),
                };
                let mut right = match right.eval(context) {
                    V::Sum(constructors) => constructors,
                    otherwise => panic!("Not a Sum expression: `{}`.", otherwise),
                };
                left.append(&mut right);
                V::Sum(left)
            }
            E::Split(case_tree) => V::Split(branch_to_righted(case_tree, context)),
            E::Pi(input, output) => {
                let pattern = input.pattern;
                let input = Box::new(input.expression.eval(context.clone()));
                let extra_info = Some(input.clone());
                let output = Closure::Abstraction(pattern, extra_info, *output, Box::new(context));
                V::Pi(input, output)
            }
            E::Sigma(first, second) => {
                let pattern = first.pattern;
                let first = Box::new(first.expression.eval(context.clone()));
                let extra_info = Some(first.clone());
                let second = Closure::Abstraction(pattern, extra_info, *second, Box::new(context));
                V::Sigma(first, second)
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
