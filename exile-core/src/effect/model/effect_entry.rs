use crate::{effect::EffectConditionEvaluator, game::Game};

#[derive(Debug, PartialEq)]
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
