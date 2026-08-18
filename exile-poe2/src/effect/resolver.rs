use exile_core::effect::{EffectEntry, ModifierEffectResolver};

use crate::item::state::{
    EquipSlot, Poe2, Poe2Effect, Poe2ModifierDefinition, Poe2ModifierInstance, Poe2StatModifierKind,
};

use std::cell::Cell;

pub struct Poe2ModifierEffectResolver {
    pub current_slot: Cell<EquipSlot>,
}

impl Poe2ModifierEffectResolver {
    pub fn new() -> Self {
        Self {
            current_slot: Cell::new(EquipSlot::None),
        }
    }
}

impl Default for Poe2ModifierEffectResolver {
    fn default() -> Self {
        Self::new()
    }
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
            match &stat_def.kind {
                Poe2StatModifierKind::Plain => {
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

                Poe2StatModifierKind::Conditional(condition) => {
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
                    effects.push(EffectEntry::conditional(effect, *condition));
                }

                Poe2StatModifierKind::Scaled(scaling) => {
                    let effect = Poe2Effect::ScaledStat {
                        target_id: stat_def.id_hash,
                        multiplier: roll,
                        scaling: *scaling,
                    };
                    effects.push(EffectEntry::unconditional(effect));
                }
            }
        }

        Ok(effects)
    }
}
