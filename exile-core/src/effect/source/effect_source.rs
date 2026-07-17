use crate::{effect::EffectEntry, game::Game};

pub trait EffectSource<G>
where
    G: Game,
{
    fn effect_source_id(&self) -> G::EffectSourceId;

    fn collect_effects(&self) -> Vec<EffectEntry<G>>;
}
