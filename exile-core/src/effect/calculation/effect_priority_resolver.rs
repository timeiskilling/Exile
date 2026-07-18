use crate::game::Game;

pub trait EffectPriorityResolver<G>
where
    G: Game,
{
    type Priority: Ord;

    fn priority(&self, effect: &G::Effect) -> Self::Priority;
}
