use ahash::AHashMap;

use crate::{
    ModType,
    item::state::{EquipSlot, Poe2, Poe2Effect},
};

#[derive(Default)]
pub struct LocalItemStats {
    pub stats: AHashMap<(ModType, u64), i64>,
}

#[derive(Default)]
pub struct Poe2Accumulator {
    pub global_stats: AHashMap<(ModType, u64), i64>,
    pub equipment_stats: AHashMap<EquipSlot, LocalItemStats>,
    pub pending_scaling: Vec<Poe2Effect>,
}
pub struct Poe2EffectApplier;

impl exile_core::effect::EffectApplier<Poe2> for Poe2EffectApplier {
    type Accumulator = Poe2Accumulator;
    type Error = std::convert::Infallible;

    fn apply_effect(
        &self,
        effect: &Poe2Effect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        match effect {
            Poe2Effect::GlobalStat {
                id,
                value,
                mod_type,
            } => {
                *accumulator
                    .global_stats
                    .entry((*mod_type, *id))
                    .or_insert(0) += value;
            }
            Poe2Effect::LocalStat {
                slot,
                id,
                value,
                mod_type,
            } => {
                let local_pool = accumulator.equipment_stats.entry(*slot).or_default();
                *local_pool.stats.entry((*mod_type, *id)).or_insert(0) += value;
            }
            Poe2Effect::ScaledStat { .. } => {
                accumulator.pending_scaling.push(*effect);
            }
        }
        Ok(())
    }
}
