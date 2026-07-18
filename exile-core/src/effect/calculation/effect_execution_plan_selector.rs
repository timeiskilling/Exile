use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use crate::{
    effect::calculation::{EffectExecutionPlan, EffectStrengthResolver},
    game::Game,
};

pub struct EffectExecutionPlanSelector<R> {
    strength_resolver: R,
}

impl<R> EffectExecutionPlanSelector<R> {
    pub fn new(strength_resolver: R) -> Self {
        Self { strength_resolver }
    }

    pub fn select<'a, G>(&self, plan: EffectExecutionPlan<'a, G>) -> EffectExecutionPlan<'a, G>
    where
        G: Game,
        R: EffectStrengthResolver<G>,
        R::Key: Eq + Hash,
        R::Strength: Ord,
    {
        let entries = plan.into_iter().collect::<Vec<_>>();
        let mut keep = vec![true; entries.len()];

        let mut winners = HashMap::<R::Key, (R::Strength, usize)>::new();

        for (index, entry) in entries.iter().copied().enumerate() {
            let Some((key, strength)) = self.strength_resolver.strength(entry.effect()) else {
                continue;
            };

            match winners.entry(key) {
                Entry::Vacant(vacant) => {
                    vacant.insert((strength, index));
                }

                Entry::Occupied(mut occupied) => {
                    let (_, winner_index) = occupied.get();

                    if strength > occupied.get().0 {
                        keep[*winner_index] = false;
                        occupied.insert((strength, index));
                    } else {
                        keep[index] = false;
                    }
                }
            }
        }

        let selected_entries = entries
            .into_iter()
            .zip(keep)
            .filter_map(|(entry, keep)| keep.then_some(entry))
            .collect();

        EffectExecutionPlan::from_entries(selected_entries)
    }
}
