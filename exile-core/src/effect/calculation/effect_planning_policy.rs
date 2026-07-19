use std::hash::Hash;

use crate::game::Game;

pub trait EffectPlanningPolicy<G>
where
    G: Game,
{
    type Phase: Ord;
    type Priority: Ord;
    type ConflictKey: Clone + Eq + Hash;
    type SelectionKey: Eq + Hash;

    fn phase(&self, effect: &G::Effect) -> Self::Phase;

    fn priority(&self, effect: &G::Effect) -> Self::Priority;

    fn conflict_key(&self, effect: &G::Effect) -> Option<Self::ConflictKey>;

    fn selection_key(&self, effect: &G::Effect) -> Option<Self::SelectionKey>;

    fn prefers(&self, candidate: &G::Effect, current: &G::Effect) -> bool;
}
