use crate::{
    effect::{
        ActiveEffectCollection, EffectOrigin,
        calculation::{
            EffectExecutionPlan, EffectPlanner, effect_planning_policy::EffectPlanningPolicy,
            effect_selection_rejection::EffectSelectionRejection,
        },
    },
    game::Game,
};
use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
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

impl<G, K> fmt::Debug for EffectExecutionPlanValidationError<G, K>
where
    G: Game,
    K: fmt::Debug,
    EffectOrigin<G>: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingExclusiveEffects {
                key,
                first_origin,
                second_origin,
            } => formatter
                .debug_struct("ConflictingExclusiveEffects")
                .field("key", key)
                .field("first_origin", first_origin)
                .field("second_origin", second_origin)
                .finish(),
        }
    }
}

pub struct EffectExecutionPlanner<P> {
    policy: P,
}

impl<P> EffectExecutionPlanner<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }
}

impl<G, P> EffectPlanner<G> for EffectExecutionPlanner<P>
where
    G: Game,
    G::ModifierDefinitionId: Clone,
    G::EffectSourceId: Clone,
    P: EffectPlanningPolicy<G>,
{
    type Error = EffectExecutionPlanValidationError<G, P::ConflictKey>;

    fn plan<'a>(
        &self,
        effects: &ActiveEffectCollection<'a, G>,
    ) -> Result<EffectExecutionPlan<'a, G>, Self::Error> {
        let plan = EffectExecutionPlan::build(effects, &self.policy);

        validate_plan(&plan, &self.policy)?;

        Ok(select_winners(plan, &self.policy))
    }
}

fn validate_plan<G, P>(
    plan: &EffectExecutionPlan<'_, G>,
    policy: &P,
) -> Result<(), EffectExecutionPlanValidationError<G, P::ConflictKey>>
where
    G: Game,
    G::ModifierDefinitionId: Clone,
    G::EffectSourceId: Clone,
    P: EffectPlanningPolicy<G>,
{
    let mut occupied_slots = HashMap::<P::ConflictKey, EffectOrigin<G>>::new();

    for entry in plan {
        let Some(key) = policy.conflict_key(entry.effect()) else {
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

fn select_winners<'a, G, P>(
    plan: EffectExecutionPlan<'a, G>,
    policy: &P,
) -> EffectExecutionPlan<'a, G>
where
    G: Game,
    P: EffectPlanningPolicy<G>,
{
    let (entries, mut selection_rejections) = plan.into_parts();

    let mut winners = HashMap::<P::SelectionKey, usize>::new();

    for (index, entry) in entries.iter().copied().enumerate() {
        let Some(key) = policy.selection_key(entry.effect()) else {
            continue;
        };

        match winners.entry(key) {
            Entry::Vacant(vacant) => {
                vacant.insert(index);
            }

            Entry::Occupied(mut occupied) => {
                let winner_index = *occupied.get();
                let winner_entry = entries[winner_index];

                if policy.prefers(entry.effect(), winner_entry.effect()) {
                    occupied.insert(index);
                }
            }
        }
    }

    let mut selected_entries = Vec::with_capacity(entries.len());

    for (index, entry) in entries.iter().copied().enumerate() {
        let Some(key) = policy.selection_key(entry.effect()) else {
            selected_entries.push(entry);
            continue;
        };

        let winner_index = *winners
            .get(&key)
            .expect("selection group must have a winner");

        let winner_entry = entries[winner_index];

        if index == winner_index {
            selected_entries.push(entry);
        } else {
            selection_rejections.push(EffectSelectionRejection::new(entry, winner_entry));
        }
    }

    EffectExecutionPlan::from_entries(selected_entries, selection_rejections)
}
