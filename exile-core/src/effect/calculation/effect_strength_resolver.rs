use std::hash::Hash;

use crate::game::Game;

pub trait EffectStrengthResolver<G>
where
    G: Game,
{
    type Key: Eq + Hash;
    type Strength: Ord;

    fn strength(&self, effect: &G::Effect) -> Option<(Self::Key, Self::Strength)>;
}
