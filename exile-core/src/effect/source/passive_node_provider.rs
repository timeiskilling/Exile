use crate::{effect::EffectSource, game::Game};

pub trait PassiveNodeProvider<G>
where
    G: Game,
{
    type Id;
    type Node: EffectSource<G>;
    type Error;

    fn node(&self, id: &Self::Id) -> Result<&Self::Node, Self::Error>;
}
