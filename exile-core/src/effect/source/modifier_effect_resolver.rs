use crate::{effect::EffectEntry, game::Game};

pub trait ModifierEffectResolver<G>
where
    G: Game,
{
    type Error;

    fn resolve_modifier_effects(
        &self,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<Vec<EffectEntry<G>>, Self::Error>;
}
