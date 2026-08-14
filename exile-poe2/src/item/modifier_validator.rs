use ahash::HashSet;

use exile_core::item::ModifierValidator;

use crate::{item::state::Poe2, repoe_parse::HashedTagWeight};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Poe2ModifierValidationError {
    ItemLevelTooLow {
        required: u16,
        actual: u16,
    },
    RollOutsideAllowedRange {
        minimum: i64,
        maximum: i64,
        actual: i64,
    },
    MismatchedStatsCount,
    MismatchedBase,
    ModNotFound,
}

pub struct Poe2ModifierValidator;

impl Poe2ModifierValidator {
    fn effective_spawn_weight(
        &self,
        item_tags: &HashSet<u64>,
        spawn_weights: &[HashedTagWeight],
    ) -> Option<u32> {
        spawn_weights
            .iter()
            .find(|sw| item_tags.contains(&sw.tag))
            .map(|sw| sw.weight)
    }
}

impl ModifierValidator<Poe2> for Poe2ModifierValidator {
    type Error = Poe2ModifierValidationError;

    fn validate_modifier(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        definition: &<Poe2 as exile_core::game::Game>::ModifierDefinition,
        modifier: &<Poe2 as exile_core::game::Game>::ModifierInstance,
    ) -> Result<(), Self::Error> {
        if definition.stats.len() != modifier.rolls.len() {
            return Err(Poe2ModifierValidationError::MismatchedStatsCount);
        }

        match self.effective_spawn_weight(&item.state().tags, &definition.spawn_weights) {
            Some(weight) if weight > 0 => {}
            _ => return Err(Poe2ModifierValidationError::MismatchedBase),
        }

        if item.state().item_level < definition.required_level {
            return Err(Poe2ModifierValidationError::ItemLevelTooLow {
                required: definition.required_level,
                actual: item.state().item_level,
            });
        }

        for (stat_def, &rolled_value) in definition.stats.iter().zip(modifier.rolls.iter()) {
            if rolled_value < stat_def.min || rolled_value > stat_def.max {
                return Err(Poe2ModifierValidationError::RollOutsideAllowedRange {
                    minimum: stat_def.min,
                    maximum: stat_def.max,
                    actual: rolled_value,
                });
            }
        }

        Ok(())
    }
}
