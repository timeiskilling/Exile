use ahash::AHashMap;
use exile_core::effect::EffectAccumulatorFinalizer;
use std::sync::OnceLock;

use crate::{
    calculation::accumulator::Poe2Accumulator, item::state::StatBucket, repoe_parse::Requirements,
};

macro_rules! stat_buckets {
    ( $( $stat_id:expr => [ $( $bucket:ident ),+ ] ),* $(,)? ) => {{
        let mut map = AHashMap::new();
        $(
            map.insert(
                crate::item::state::hash_string($stat_id),
                vec![ $( StatBucket::$bucket ),+ ],
            );
        )*
        map
    }};
}

#[derive(Debug, Clone, Default)]
pub struct Poe2FinalStat {
    pub attributes: Attributes,
    pub resources: Resources, // Life, ES, Mana, Spirit, Rage
    pub defenses: Defenses,   // Armour, Evasion, Deflection, Block
    pub resistances: Resistances,
    pub protections: Protections, // Stun, Ailments
    pub utility: Utility,         // Flasks, Charms, Charges, Misc

    pub equipment: AHashMap<crate::item::state::EquipSlot, Poe2ItemFinalStat>,
}

#[derive(Debug, Clone, Default)]
pub struct Poe2ItemFinalStat {
    pub quality: u64,
    pub requirements: Requirements,
    // Defenses
    pub local_flat_armour: u64,
    pub local_percent_armour: f64,

    pub local_flat_evasion: u64,
    pub local_percent_evasion: f64,

    pub local_energy_shield: u64,
    pub local_percent_energy_shield: f64,

    // Offenses (Weapon)
    pub local_flat_physical_min: u64,
    pub local_flat_physical_max: u64,
    pub local_percent_physical: f64,

    pub local_flat_cold_min: u64,
    pub local_flat_cold_max: u64,
    pub local_percent_cold: f64,

    pub local_flat_lightning_min: u64,
    pub local_flat_lightning_max: u64,
    pub local_percent_lightning: f64,

    pub local_flat_fire_min: u64,
    pub local_flat_fire_max: u64,
    pub local_percent_fire: f64,

    pub local_accuracy_rating: f64,
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
    pub fn get_bucket_mapping() -> &'static AHashMap<u64, Vec<StatBucket>> {
        static MAPPING: OnceLock<AHashMap<u64, Vec<StatBucket>>> = OnceLock::new();

        MAPPING.get_or_init(|| {
            stat_buckets! {
                "base_maximum_life" => [Life],
                "maximum_life_+%" => [LifePercent],
                "base_maximum_mana" => [Mana],
                "maximum_mana_+%" => [ManaPercent],
                "chaos_damage_+%" => [ChaosDamagePercent],

                "additional_strength" => [Strength],
                "additional_dexterity" => [Dexterity],
                "additional_intelligence" => [Intelligence],

                "additional_strength_and_intelligence" => [Strength, Intelligence],
                "additional_strength_and_dexterity" => [Strength, Dexterity],
                "additional_all_attributes" => [Strength, Dexterity, Intelligence],

                "base_fire_damage_resistance_%" => [FireResistance],
                "base_cold_damage_resistance_%" => [ColdResistance],
                "base_lightning_damage_resistance_%" => [LightningResistance],
                "base_chaos_damage_resistance_%" => [ChaosResistance],

                "base_resist_all_elements_%" => [
                    FireResistance,
                    ColdResistance,
                    LightningResistance,
                    ChaosResistance
                ],

                "base_physical_damage_reduction_rating" => [Armour],
                "base_evasion_rating" => [Evasion],
                "base_maximum_energy_shield" => [EnergyShield],

                "physical_damage_reduction_rating_+%" => [ArmourPercent],
                "maximum_energy_shield_+%" => [MaximumEnergyShieldPercent],
                "stun_threshold_+" => [StunThreshold],
                "base_movement_velocity_+%" => [MovementSpeedPercent],

                "thorns_minimum_base_physical_damage" => [ThornsPhysicalMin],
                "thorns_maximum_base_physical_damage" => [ThornsPhysicalMax],

                "attack_minimum_added_physical_damage" => [GlobalAttackPhysicalMin],
                "attack_maximum_added_physical_damage" => [GlobalAttackPhysicalMax],

                "attack_minimum_added_fire_damage" => [GlobalAttackFireMin],
                "attack_maximum_added_fire_damage" => [GlobalAttackFireMax],

                "attack_minimum_added_cold_damage" => [GlobalAttackColdMin],
                "attack_maximum_added_cold_damage" => [GlobalAttackColdMax],

                "attack_minimum_added_lightning_damage" => [GlobalAttackLightningMin],
                "attack_maximum_added_lightning_damage" => [GlobalAttackLightningMax],

                "allies_in_presence_attack_minimum_added_physical_damage" => [AlliesInPresenceAttackPhysicalMin],
                "allies_in_presence_attack_maximum_added_physical_damage" => [AlliesInPresenceAttackPhysicalMax],
                "allies_in_presence_attack_minimum_added_fire_damage" => [AlliesInPresenceAttackFireMin],
                "allies_in_presence_attack_maximum_added_fire_damage" => [AlliesInPresenceAttackFireMax],
                "allies_in_presence_attack_minimum_added_cold_damage" => [AlliesInPresenceAttackColdMin],
                "allies_in_presence_attack_maximum_added_cold_damage" => [AlliesInPresenceAttackColdMax],
                "allies_in_presence_attack_minimum_added_lightning_damage" => [AlliesInPresenceAttackLightningMin],
                "allies_in_presence_attack_maximum_added_lightning_damage" => [AlliesInPresenceAttackLightningMax],
                "allies_in_presence_attack_minimum_added_chaos_damage" => [AlliesInPresenceAttackChaosMin],
                "allies_in_presence_attack_maximum_added_chaos_damage" => [AlliesInPresenceAttackChaosMax],
                "allies_in_presence_damage_+%" => [AlliesInPresenceDamageIncreasePrecent],
                "local_accuracy_rating" => [AccuracyRating],

                "chance_for_exerted_attacks_to_not_reduce_count_%" => [ChanceToNotConsumeExertedAttack],
            }
        })
    }
}

impl EffectAccumulatorFinalizer for Poe2Finalizer {
    type Accumulator = Poe2Accumulator;
    type Output = Poe2FinalStat;
    type Error = std::convert::Infallible;

    fn finalize(&self, acc: Poe2Accumulator) -> Result<Poe2FinalStat, Self::Error> {
        let mut final_stat = Poe2FinalStat::default();
        let buckets_by_id = Self::get_bucket_mapping();

        let bucket_totals = self.finalize_by_bucket(&acc, buckets_by_id);

        // Example of applying a bucket total to the final stat:
        if let Some(&_chaos_dmg) = bucket_totals.get(&StatBucket::ChaosDamagePercent) {
            // final_stat.offense.chaos_damage_percent += chaos_dmg;
        }

        // 2. Resolve global stats via the stat_balancer.rs ModType definitions (for complex/conditional mods)
        for ((mod_type, stat_id), value) in acc.global_stats.iter() {
            mod_type.resolve_modifier(*stat_id, *value, None, &mut final_stat);
        }

        // 3. Resolve equipment local stats, passing the slot
        for (slot, local_pool) in acc.equipment_stats.iter() {
            for ((mod_type, stat_id), value) in local_pool.stats.iter() {
                mod_type.resolve_modifier(*stat_id, *value, Some(*slot), &mut final_stat);
            }
        }

        Ok(final_stat)
    }
}
