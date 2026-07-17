use crate::{
    effect::{EffectConditionEvaluator, EffectEntry, model::EffectOrigin},
    game::Game,
};

pub struct SourcedEffectEntry<G>
where
    G: Game,
{
    entry: EffectEntry<G>,
    origin: EffectOrigin<G>,
}

impl<G> SourcedEffectEntry<G>
where
    G: Game,
{
    pub fn new(entry: EffectEntry<G>, origin: EffectOrigin<G>) -> Self {
        Self { entry, origin }
    }

    pub fn entry(&self) -> &EffectEntry<G> {
        &self.entry
    }

    pub fn effect(&self) -> &G::Effect {
        self.entry.effect()
    }

    pub fn condition(&self) -> Option<&G::EffectCondition> {
        self.entry.condition()
    }

    pub fn origin(&self) -> &EffectOrigin<G> {
        &self.origin
    }

    pub fn is_active<E>(&self, evaluator: &E, context: &E::Context) -> Result<bool, E::Error>
    where
        E: EffectConditionEvaluator<G>,
    {
        self.entry.is_active(evaluator, context)
    }

    pub fn into_parts(self) -> (EffectEntry<G>, EffectOrigin<G>) {
        (self.entry, self.origin)
    }
}
