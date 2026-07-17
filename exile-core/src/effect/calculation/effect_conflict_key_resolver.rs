use crate::game::Game;

pub trait EffectConflictKeyResolver<G>
where
    G: Game,
{
    type Key;

    fn conflict_key(&self, effect: &G::Effect) -> Option<Self::Key>;
}
