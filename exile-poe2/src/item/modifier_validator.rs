use exile_core::item::ModifierValidator;

use crate::item::state::Poe2;

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
}

pub struct Poe2ModifierValidator;

impl ModifierValidator<Poe2> for Poe2ModifierValidator {
    type Error = Poe2ModifierValidationError;

    fn validate_modifier(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        definition: &<Poe2 as exile_core::game::Game>::ModifierDefinition,
        modifier: &<Poe2 as exile_core::game::Game>::ModifierInstance,
    ) -> Result<(), Self::Error> {
        if definition.stats.len() != modifier.len() {
            return Err(Poe2ModifierValidationError::MismatchedStatsCount);
        }

        if item.state().item_level < definition.required_level {
            return Err(Poe2ModifierValidationError::ItemLevelTooLow {
                required: definition.required_level,
                actual: item.state().item_level,
            });
        }
        
        for (stat_def, &rolled_value) in definition.stats.iter().zip(modifier.iter()) {
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
