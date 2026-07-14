use crate::{
    effect::{
        active_effect_collection::ActiveEffectCollection, effect_collection::EffectCollection,
        effect_condition_evaluator::EffectConditionEvaluator,
    },
    game::Game,
};

pub struct EffectCollectionEvaluator<E> {
    condition_evaluator: E,
}

impl<E> EffectCollectionEvaluator<E> {
    pub fn new(condition_evaluator: E) -> Self {
        Self {
            condition_evaluator,
        }
    }

    pub fn collect_active<'a, G>(
        &self,
        collection: &'a EffectCollection<G>,
        context: &<E as EffectConditionEvaluator<G>>::Context,
    ) -> Result<ActiveEffectCollection<'a, G>, <E as EffectConditionEvaluator<G>>::Error>
    where
        G: Game,
        E: EffectConditionEvaluator<G>,
    {
        let mut active_entries = Vec::new();

        for entry in collection {
            if entry.is_active(&self.condition_evaluator, context)? {
                active_entries.push(entry);
            }
        }

        Ok(ActiveEffectCollection::new(active_entries))
    }
}
