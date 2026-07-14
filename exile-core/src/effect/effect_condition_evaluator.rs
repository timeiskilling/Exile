use crate::game::Game;

pub trait EffectConditionEvaluator<G>
where
    G: Game,
{
    type Context;
    type Error;

    fn evaluate_condition(
        &self,
        condition: &G::EffectCondition,
        context: &Self::Context,
    ) -> Result<bool, Self::Error>;
}
