use crate::game::Game;

pub trait EffectApplier<G>
where
    G: Game,
{
    type Accumulator;
    type Error;

    fn apply_effect(
        &self,
        effect: &G::Effect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error>;
}
