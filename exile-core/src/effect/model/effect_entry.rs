use std::fmt;

use crate::{effect::EffectConditionEvaluator, game::Game};

pub struct EffectEntry<G>
where
    G: Game,
{
    effect: G::Effect,
    condition: Option<G::EffectCondition>,
}

impl<G> EffectEntry<G>
where
    G: Game,
{
    pub fn unconditional(effect: G::Effect) -> Self {
        Self {
            effect,
            condition: None,
        }
    }

    pub fn conditional(effect: G::Effect, condition: G::EffectCondition) -> Self {
        Self {
            effect,
            condition: Some(condition),
        }
    }

    pub fn effect(&self) -> &G::Effect {
        &self.effect
    }

    pub fn condition(&self) -> Option<&G::EffectCondition> {
        self.condition.as_ref()
    }

    pub fn is_active<E>(&self, evaluator: &E, context: &E::Context) -> Result<bool, E::Error>
    where
        E: EffectConditionEvaluator<G>,
    {
        let Some(condition) = self.condition.as_ref() else {
            return Ok(true);
        };

        evaluator.evaluate_condition(condition, context)
    }

    pub fn into_effect(self) -> G::Effect {
        self.effect
    }
}

impl<G> fmt::Debug for EffectEntry<G>
where
    G: Game,
    G::Effect: fmt::Debug,
    G::EffectCondition: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EffectEntry")
            .field("effect", &self.effect)
            .field("condition", &self.condition)
            .finish()
    }
}

impl<G> PartialEq for EffectEntry<G>
where
    G: Game,
    G::Effect: PartialEq,
    G::EffectCondition: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect && self.condition == other.condition
    }
}
