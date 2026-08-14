use std::collections::HashMap;

use crate::item::state::{Poe2, Poe2Effect};

pub struct Poe2Accumulator {
    pub stats: HashMap<u64, i64>,
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
            Poe2Effect::Stats { id, value } => {
                *accumulator.stats.entry(*id).or_insert(0) += value;
            }
        }
        Ok(())
    }
}
