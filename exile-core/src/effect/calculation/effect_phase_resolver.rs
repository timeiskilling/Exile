use crate::game::Game;

pub trait EffectPhaseResolver<G>
where
    G: Game,
{
    type Phase: Ord;

    fn phase(&self, effect: &G::Effect) -> Self::Phase;
}
