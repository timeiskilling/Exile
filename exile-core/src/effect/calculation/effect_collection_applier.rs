use crate::{
    effect::{ActiveEffectCollection, EffectApplier},
    game::Game,
};

pub struct EffectCollectionApplier<A> {
    effect_applier: A,
}

impl<A> EffectCollectionApplier<A> {
    pub fn new(effect_applier: A) -> Self {
        Self { effect_applier }
    }

    pub fn apply_all<G>(
        &self,
        effects: &ActiveEffectCollection<'_, G>,
        accumulator: &mut <A as EffectApplier<G>>::Accumulator,
    ) -> Result<(), <A as EffectApplier<G>>::Error>
    where
        G: Game,
        A: EffectApplier<G>,
    {
        for effect in effects.effects() {
            self.effect_applier.apply_effect(effect, accumulator)?;
        }

        Ok(())
    }
}
