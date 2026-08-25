use ahash::AHashMap;
use exile_core::effect::EffectAccumulatorFinalizer;

use crate::{calculation::accumulator::Poe2Accumulator, item::state::StatBucket};

#[derive(Debug, Clone, Default)]
pub struct Poe2FinalStat {
    pub attributes: Attributes,
    pub resources: Resources, // Life, ES, Mana, Spirit, Rage
    pub defenses: Defenses,   // Armour, Evasion, Deflection, Block
    pub resistances: Resistances,
    pub protections: Protections, // Stun, Ailments
    pub utility: Utility,         // Flasks, Charms, Charges, Misc
}

#[derive(Debug, Clone, Default)]
pub struct Attributes {
    pub strength: u64,
    pub dexterity: u64,
    pub intelligence: u64,
}

#[derive(Debug, Clone, Default)]
pub struct Resources {
    pub life: LifePool,
    pub energy_shield: EnergyShieldPool,
    pub mana: ManaPool,
    pub spirit: u64,
    pub rage: RagePool,
    pub accuracy: i64,
}

#[derive(Debug, Clone, Default)]
pub struct LifePool {
    pub maximum: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ManaPool {
    pub maximum: u64,
    pub recovery_per_second: RecoveryBreakdown,
}

#[derive(Debug, Clone, Default)]
pub struct RagePool {
    pub maximum: u64,
    pub inherent_loss_delay_seconds: f64,
    pub inherent_loss_per_second: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoverySource {
    Regeneration,
    Flask,
    Leech,
    Other,
}

#[derive(Debug, Clone, Default)]
pub struct RecoveryBreakdown {
    pub total: f64,
    pub by_source: AHashMap<RecoverySource, f64>,
}

#[derive(Debug, Clone, Default)]
pub struct Defenses {
    pub armour: DefenseValue,
    pub evasion: DefenseValue,
    pub deflection: DeflectionValue,
    pub block_chance_percent: f64,
}

#[derive(Debug, Clone, Default)]
pub struct DefenseValue {
    pub rating: u64,
    pub estimated_percent: f64,

    pub from_items: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct EnergyShieldPool {
    pub maximum: u64,
    pub recharge_per_second: f64,
    pub recharge_delay_seconds: f64,

    pub from_items: Option<u64>,
}
#[derive(Debug, Clone, Default)]
pub struct DeflectionValue {
    pub rating: u64,
    pub estimated_chance_percent: f64,
    pub damage_prevented_percent: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Resistances {
    pub fire: Resistance,
    pub cold: Resistance,
    pub lightning: Resistance,
    pub chaos: Resistance,
}

#[derive(Debug, Clone, Default)]
pub struct Resistance {
    pub effective_percent: f64,
    pub uncapped_percent: f64,
    pub max_percent: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Protections {
    pub stun_recovery_modifier_percent: f64,
    pub elemental_ailment_threshold: u64,

    pub immunities: Vec<String>,
    pub ailment_duration_on_self: AilmentDurations,
}

#[derive(Debug, Clone, Default)]
pub struct AilmentDurations {
    pub ignite_modifier_percent: f64,
    pub chill_modifier_percent: f64,
    pub freeze_modifier_percent: f64,
    pub shock_modifier_percent: f64,
    pub poison_modifier_percent: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Utility {
    pub flask_mana_amount_modifier_percent: f64,
    pub charm_duration_modifier_percent: f64,
    pub charges: Charges,
    pub movement_speed_modifier_percent: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Charges {
    pub power_maximum: u64,
    pub frenzy_maximum: u64,
    pub endurance_maximum: u64,
}

pub struct Poe2Finalizer;

impl Poe2Finalizer {
    fn add_stats_to_buckets(
        &self,
        stats: &AHashMap<(crate::ModType, u64), i64>,
        buckets_by_id: &AHashMap<u64, Vec<StatBucket>>,
        totals: &mut AHashMap<StatBucket, i64>,
    ) {
        for ((_mod_type, id), value) in stats {
            if let Some(buckets) = buckets_by_id.get(id) {
                for bucket in buckets {
                    *totals.entry(*bucket).or_insert(0) += value;
                }
            }
        }
    }

    fn finalize_by_bucket(
        &self,
        acc: &Poe2Accumulator,
        buckets_by_id: &AHashMap<u64, Vec<StatBucket>>,
    ) -> AHashMap<StatBucket, i64> {
        let mut totals = AHashMap::default();

        // Accumulate global stats
        for ((_mod_type, stat_id), value) in &acc.global_stats {
            if let Some(buckets) = buckets_by_id.get(stat_id) {
                for bucket in buckets {
                    *totals.entry(*bucket).or_insert(0) += value;
                }
            }
        }

        // Accumulate local equipment stats
        for (_slot, local_pool) in &acc.equipment_stats {
            for ((_mod_type, stat_id), value) in &local_pool.stats {
                if let Some(buckets) = buckets_by_id.get(stat_id) {
                    for bucket in buckets {
                        *totals.entry(*bucket).or_insert(0) += value;
                    }
                }
            }
        }

        totals
    }
    pub fn default_bucket_mapping() -> AHashMap<u64, Vec<StatBucket>> {
        let mut map = AHashMap::new();

        map.insert(
            crate::item::state::hash_string("base_maximum_life"),
            vec![StatBucket::Life],
        );

        map.insert(
            crate::item::state::hash_string("maximum_life_+%"),
            vec![StatBucket::LifePercent],
        );

        map.insert(
            crate::item::state::hash_string("base_maximum_mana"),
            vec![StatBucket::Mana],
        );

        map.insert(
            crate::item::state::hash_string("maximum_mana_+%"),
            vec![StatBucket::ManaPercent],
        );
        map.insert(
            crate::item::state::hash_string("chaos_damage_+%"),
            vec![StatBucket::ChaosDamagePercent],
        );
        map.insert(
            crate::item::state::hash_string("additional_strength"),
            vec![StatBucket::Strength],
        );
        map.insert(
            crate::item::state::hash_string("additional_strength_and_intelligence"),
            vec![StatBucket::Strength, StatBucket::Intelligence],
        );
        map.insert(
            crate::item::state::hash_string("additional_strength_and_dexterity"),
            vec![StatBucket::Strength, StatBucket::Dexterity],
        );
        map.insert(
            crate::item::state::hash_string("additional_all_attributes"),
            vec![
                StatBucket::Strength,
                StatBucket::Dexterity,
                StatBucket::Intelligence,
            ],
        );

        map.insert(
            crate::item::state::hash_string("base_fire_damage_resistance_%"),
            vec![StatBucket::FireResistance],
        );

        map.insert(
            crate::item::state::hash_string("base_cold_damage_resistance_%"),
            vec![StatBucket::ColdResistance],
        );

        map.insert(
            crate::item::state::hash_string("base_lightning_damage_resistance_%"),
            vec![StatBucket::LightningResistance],
        );

        map.insert(
            crate::item::state::hash_string("base_resist_all_elements_%"),
            vec![
                StatBucket::FireResistance,
                StatBucket::ColdResistance,
                StatBucket::LightningResistance,
                StatBucket::ChaosResistance,
            ],
        );

        map.insert(
            crate::item::state::hash_string("base_chaos_damage_resistance_%"),
            vec![StatBucket::ChaosResistance],
        );

        map
    }
}

impl EffectAccumulatorFinalizer for Poe2Finalizer {
    type Accumulator = Poe2Accumulator;
    type Output = Poe2FinalStat;
    type Error = std::convert::Infallible;

    fn finalize(&self, acc: Poe2Accumulator) -> Result<Poe2FinalStat, Self::Error> {
        let mut final_stat = Poe2FinalStat::default();
        let buckets_by_id = Self::default_bucket_mapping();

        let bucket_totals = self.finalize_by_bucket(&acc, &buckets_by_id);

        // Example of applying a bucket total to the final stat:
        if let Some(&_chaos_dmg) = bucket_totals.get(&StatBucket::ChaosDamagePercent) {
            // final_stat.offense.chaos_damage_percent += chaos_dmg;
        }

        // 2. Resolve global stats via the stat_balancer.rs ModType definitions (for complex/conditional mods)
        for ((mod_type, _stat_id), value) in acc.global_stats.iter() {
            mod_type.resolve_modifier(*value, None, &mut final_stat);
        }

        // 3. Resolve equipment local stats, passing the slot
        for (slot, local_pool) in acc.equipment_stats.iter() {
            for ((mod_type, _stat_id), value) in local_pool.stats.iter() {
                mod_type.resolve_modifier(*value, Some(*slot), &mut final_stat);
            }
        }

        Ok(final_stat)
    }
}
