use crate::{effect::effect_entry::EffectEntry, game::Game};

pub trait EffectSource<G>
where
    G: Game,
{
    fn collect_effects(&self) -> Vec<EffectEntry<G>>;
}
