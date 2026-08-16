use std::collections::HashSet;

use exile_core::effect::EffectConditionEvaluator;

use crate::{item::state::Poe2, poe2_condition::Poe2Condition};

#[derive(Debug, Default, Clone)]
pub struct Poe2Configuration {
    pub active_conditions: HashSet<Poe2Condition>,
}

pub struct Poe2Evaluator;

impl EffectConditionEvaluator<Poe2> for Poe2Evaluator {
    type Context = Poe2Configuration;
    type Error = std::convert::Infallible;

    fn evaluate_condition(
        &self,
        condition: &Poe2Condition,
        context: &Self::Context,
    ) -> Result<bool, Self::Error> {
        Ok(context.active_conditions.contains(condition))
    }
}
