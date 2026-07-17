use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use crate::{
    effect::{
        EffectOrigin,
        calculation::{EffectConflictKeyResolver, EffectExecutionPlan},
    },
    game::Game,
};

pub enum EffectExecutionPlanValidationError<G, K>
where
    G: Game,
{
    ConflictingExclusiveEffects {
        key: K,
        first_origin: EffectOrigin<G>,
        second_origin: EffectOrigin<G>,
    },
}

pub struct EffectExecutionPlanValidator<R> {
    conflict_key_resolver: R,
}

impl<R> EffectExecutionPlanValidator<R> {
    pub fn new(conflict_key_resolver: R) -> Self {
        Self {
            conflict_key_resolver,
        }
    }

    pub fn validate<G>(
        &self,
        plan: &EffectExecutionPlan<'_, G>,
    ) -> Result<(), EffectExecutionPlanValidationError<G, <R as EffectConflictKeyResolver<G>>::Key>>
    where
        G: Game,
        G::ModifierDefinitionId: Clone,
        G::EffectSourceId: Clone,
        R: EffectConflictKeyResolver<G>,
        R::Key: Clone + Eq + Hash,
    {
        let mut occupied_slots = HashMap::<R::Key, EffectOrigin<G>>::new();

        for entry in plan {
            let Some(key) = self.conflict_key_resolver.conflict_key(entry.effect()) else {
                continue;
            };

            match occupied_slots.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert(entry.origin().clone());
                }

                Entry::Occupied(occupied) => {
                    return Err(
                        EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                            key: occupied.key().clone(),
                            first_origin: occupied.get().clone(),
                            second_origin: entry.origin().clone(),
                        },
                    );
                }
            }
        }

        Ok(())
    }
}
