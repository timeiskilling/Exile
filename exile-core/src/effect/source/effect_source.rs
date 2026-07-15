use crate::{effect::EffectEntry, game::Game};

pub trait EffectSource<G>
where
    G: Game,
{
    fn collect_effects(&self) -> Vec<EffectEntry<G>>;
}
