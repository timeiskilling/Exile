use ahash::AHashMap;
use exile_core::effect::EffectAccumulatorFactory;

use crate::{calculation::accumulator::Poe2Accumulator, item::base::Poe2CharacterBase};

pub struct Poe2AccumulatorFactory;

impl EffectAccumulatorFactory for Poe2AccumulatorFactory {
    type Input = Poe2CharacterBase;
    type Accumulator = Poe2Accumulator;
    type Error = std::convert::Infallible;

    fn create(&self, input: &Self::Input) -> Result<Self::Accumulator, Self::Error> {
        let mut global_stats = AHashMap::new();

        global_stats.insert(
            crate::item::state::hash_string("base_maximum_life"),
            input.base_life,
        );
        global_stats.insert(
            crate::item::state::hash_string("base_maximum_mana"),
            input.base_mana,
        );
        global_stats.insert(
            crate::item::state::hash_string("additional_strength"),
            input.base_strength,
        );
        global_stats.insert(
            crate::item::state::hash_string("additional_dexterity"),
            input.base_dexterity,
        );
        global_stats.insert(
            crate::item::state::hash_string("additional_intelligence"),
            input.base_intelligence,
        );

        let equipment_stats = AHashMap::new();

        Ok(Poe2Accumulator {
            global_stats,
            equipment_stats,
            pending_scaling: Vec::new(),
        })
    }
}
