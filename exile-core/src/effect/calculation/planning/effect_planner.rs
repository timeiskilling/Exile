use crate::{
    effect::{ActiveEffectCollection, calculation::EffectExecutionPlan},
    game::Game,
};

pub trait EffectPlanner<G>
where
    G: Game,
{
    type Error;

    fn plan<'a>(
        &self,
        effects: &ActiveEffectCollection<'a, G>,
    ) -> Result<EffectExecutionPlan<'a, G>, Self::Error>;
}
