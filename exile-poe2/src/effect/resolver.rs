use exile_core::effect::{EffectEntry, ModifierEffectResolver};

use crate::item::state::{
    EquipSlot, Poe2, Poe2Effect, Poe2ModifierDefinition, Poe2ModifierInstance,
};

use std::cell::Cell;

pub struct Poe2ModifierEffectResolver {
    pub current_slot: Cell<EquipSlot>,
}

impl ModifierEffectResolver<Poe2> for Poe2ModifierEffectResolver {
    type Error = std::convert::Infallible;

    fn resolve_modifier_effects(
        &self,
        definition: &Poe2ModifierDefinition,
        modifier: &Poe2ModifierInstance,
    ) -> Result<Vec<EffectEntry<Poe2>>, Self::Error> {
        let mut effects = Vec::new();
        for (stat_def, &roll) in definition.stats.iter().zip(modifier.rolls.iter()) {
            let effect = if stat_def.is_local {
                Poe2Effect::LocalStat {
                    slot: self.current_slot.get(),
                    id: stat_def.id_hash,
                    value: roll,
                }
            } else {
                Poe2Effect::GlobalStat {
                    id: stat_def.id_hash,
                    value: roll,
                }
            };

            effects.push(EffectEntry::unconditional(effect));
        }

        Ok(effects)
    }
}
