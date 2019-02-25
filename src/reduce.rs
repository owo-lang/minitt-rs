use crate::syntax::Expression;
use crate::syntax::Telescope;
use crate::syntax::Value;

/// Evaluate an [`Expression`] to a [`Value`] under a [`Telescope`]
pub fn eval<Name>(expr: Expression<Name>, context: Telescope<Name>) -> Value<Name>
where
    Name: Eq,
    Name: Clone,
{
    unimplemented!()
}
