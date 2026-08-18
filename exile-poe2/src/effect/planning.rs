use ahash::AHashMap;
use exile_core::effect::EffectPlanningPolicy;

use crate::item::definition::Poe2DefinitionRegistry;
use crate::item::state::{Poe2, Poe2Effect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Poe2EffectPhase {
    Base,
    AddedFlat,
    Conversion,
    IncreasedReduced,
    MoreLess,
    CapsAndLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Poe2EffectPriority {
    Inherent,
    Normal,
    Late,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Poe2ConflictKey {
    ChaosInoculationLifeOverride,
    AvatarOfFireDamageRestriction,
    ResoluteTechniqueCritOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Poe2SelectionKey {
    Aura(u64),
    ActionSpeedFloor,
    MinimumFrenzyCharges,
}

pub struct Poe2PlanningPolicy {
    phase_by_id: AHashMap<u64, Poe2EffectPhase>,
    conflict_by_id: AHashMap<u64, Poe2ConflictKey>,
    selection_by_id: AHashMap<u64, Poe2SelectionKey>,
}

impl Poe2PlanningPolicy {
    pub fn from_registry(registry: &Poe2DefinitionRegistry) -> Self {
        let mut phase_by_id = AHashMap::new();
        let mut conflict_by_id = AHashMap::new();
        let mut selection_by_id = AHashMap::new();

        for def in registry.definitions.values() {
            for stat in &def.stats {
                phase_by_id.insert(stat.id_hash, stat.phase);

                if let Some(conflict) = stat.conflict_key {
                    conflict_by_id.insert(stat.id_hash, conflict);
                }

                if let Some(selection) = stat.selection_key {
                    selection_by_id.insert(stat.id_hash, selection);
                }
            }
        }

        Self {
            phase_by_id,
            conflict_by_id,
            selection_by_id,
        }
    }

    fn effect_id(effect: &Poe2Effect) -> u64 {
        match effect {
            Poe2Effect::GlobalStat { id, .. } | Poe2Effect::LocalStat { id, .. } => *id,
            Poe2Effect::ScaledStat { target_id, .. } => *target_id,
        }
    }
}

impl EffectPlanningPolicy<Poe2> for Poe2PlanningPolicy {
    type Phase = Poe2EffectPhase;
    type Priority = Poe2EffectPriority;
    type ConflictKey = Poe2ConflictKey;
    type SelectionKey = Poe2SelectionKey;

    fn phase(&self, effect: &Poe2Effect) -> Self::Phase {
        match effect {
            Poe2Effect::GlobalStat { id, .. } | Poe2Effect::LocalStat { id, .. } => self
                .phase_by_id
                .get(id)
                .copied()
                .unwrap_or(Poe2EffectPhase::AddedFlat),
            Poe2Effect::ScaledStat { .. } => Poe2EffectPhase::Base,
        }
    }

    fn priority(&self, _effect: &Poe2Effect) -> Self::Priority {
        Poe2EffectPriority::Normal
    }

    fn conflict_key(&self, effect: &Poe2Effect) -> Option<Self::ConflictKey> {
        self.conflict_by_id.get(&Self::effect_id(effect)).copied()
    }

    fn selection_key(&self, effect: &Poe2Effect) -> Option<Self::SelectionKey> {
        self.selection_by_id.get(&Self::effect_id(effect)).copied()
    }

    fn prefers(&self, candidate: &Poe2Effect, current: &Poe2Effect) -> bool {
        match (candidate, current) {
            (
                Poe2Effect::GlobalStat { value: v1, .. },
                Poe2Effect::GlobalStat { value: v2, .. },
            ) => v1 > v2,
            (Poe2Effect::LocalStat { value: v1, .. }, Poe2Effect::LocalStat { value: v2, .. }) => {
                v1 > v2
            }
            (
                Poe2Effect::ScaledStat { multiplier: v1, .. },
                Poe2Effect::ScaledStat { multiplier: v2, .. },
            ) => v1 > v2,
            _ => false,
        }
    }
}
