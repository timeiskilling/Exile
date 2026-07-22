use crate::{effect::EffectCollection, game::Game};

pub trait BuildEffectCollector<G>
where
    G: Game,
{
    type Build;
    type Error;

    fn collect_effects(&self, build: &Self::Build) -> Result<EffectCollection<G>, Self::Error>;
}
