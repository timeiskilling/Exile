use exile_core::effect::{EffectEntry, ModifierEffectResolver};

use crate::item::state::{Poe2, Poe2Effect, Poe2ModifierDefinition, Poe2ModifierInstance};

pub struct Poe2ModifierEffectResolver;

impl ModifierEffectResolver<Poe2> for Poe2ModifierEffectResolver {
    type Error = std::convert::Infallible;

    fn resolve_modifier_effects(
        &self,
        definition: &Poe2ModifierDefinition,
        modifier: &Poe2ModifierInstance,
    ) -> Result<Vec<EffectEntry<Poe2>>, Self::Error> {
        let mut effects = Vec::new();
        for (stat_def, &roll) in definition.stats.iter().zip(modifier.rolls.iter()) {
            let effect = Poe2Effect::Stats {
                id: stat_def.id_hash,
                value: roll,
            };

            effects.push(EffectEntry::unconditional(effect));
        }

        Ok(effects)
    }
}
