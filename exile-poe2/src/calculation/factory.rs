use ahash::AHashMap;
use exile_core::effect::EffectAccumulatorFactory;

use crate::{
    ModType,
    calculation::accumulator::{LocalItemStats, Poe2Accumulator},
    item::{
        base::Poe2CharacterBase,
        state::{EquipSlot, StatBucket},
    },
    repoe_parse::Properties,
};

#[derive(Debug, Clone)]
pub struct Poe2CalculationInput {
    pub character: Poe2CharacterBase,
    pub equipment_properties: AHashMap<EquipSlot, Properties>,
}

pub struct Poe2AccumulatorFactory;

impl EffectAccumulatorFactory for Poe2AccumulatorFactory {
    type Input = Poe2CalculationInput;
    type Accumulator = Poe2Accumulator;
    type Error = std::convert::Infallible;

    fn create(&self, input: &Self::Input) -> Result<Self::Accumulator, Self::Error> {
        let mut global_stats = AHashMap::new();

        global_stats.insert(
            (
                ModType::BaseLifeAndMana,
                crate::item::state::hash_string("base_maximum_life"),
            ),
            input.character.base_life,
        );
        global_stats.insert(
            (
                ModType::BaseLifeAndMana,
                crate::item::state::hash_string("base_maximum_mana"),
            ),
            input.character.base_mana,
        );
        global_stats.insert(
            (
                ModType::Strength,
                crate::item::state::hash_string("additional_strength"),
            ),
            input.character.base_strength,
        );
        global_stats.insert(
            (
                ModType::Dexterity,
                crate::item::state::hash_string("additional_dexterity"),
            ),
            input.character.base_dexterity,
        );
        global_stats.insert(
            (
                ModType::Intelligence,
                crate::item::state::hash_string("additional_intelligence"),
            ),
            input.character.base_intelligence,
        );

        let mut equipment_stats = AHashMap::new();
        for (slot, props) in &input.equipment_properties {
            equipment_stats.insert(
                *slot,
                LocalItemStats {
                    properties: props.clone(),
                    stats: AHashMap::new(),
                },
            );
        }

        Ok(Poe2Accumulator {
            global_stats,
            equipment_stats,
            pending_scaling: Vec::new(),
        })
    }
}
