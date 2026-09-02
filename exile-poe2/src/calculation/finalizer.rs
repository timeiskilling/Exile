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
    pub local_block_chance: u64,

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

    pub local_life_leech_from_physical_damage: f64,
    pub local_mana_leech_from_physical_damage: f64,

    pub local_life_gain_per_hit: u64,
    pub local_mana_gain_per_hit: u64,
    pub local_attack_speed: f64,

    pub local_critical_strike_chance: f64,
    pub local_critical_strike_multiplier: i64,

    pub local_spirit_percent_increase: u64,
    pub number_of_additional_arrows: u64,
    pub chance_to_fire_1_additional_projectile: u64,

    pub number_of_additional_charm_slots: u64,
    pub local_base_stun_duration: u64,
    pub local_hit_damage_stun_multiplier: u64,

    pub local_flask_charges_gained: u64,
    pub local_flask_max_charges: u64,
    pub local_flask_charges_reduced_used: i64,
    pub local_chance_to_gain_flask_charge_on_kill: u64,
    pub local_flask_gain_x_charges_every_minute: u64,
    pub local_flask_recovery_speed_plus_percent: u64,
    pub local_flask_amount_to_recover_plus_percent: i64,
    pub local_flask_amount_to_recover_plus_percent_when_on_low_life: u64,
    pub local_flask_amount_to_recover_plus_percent_when_on_low_mana: u64,
    pub local_flask_life_to_recover_plus_percent: u64,
    pub local_flask_mana_to_recover_plus_percent: u64,
    pub local_flask_removes_of_life_recovery_from_mana_on_use: f64,
    pub local_flask_removes_of_mana_recovery_from_life_on_use: f64,
    pub local_flask_recover_instantly_percent: f64,
    pub local_flask_recover_instantly: bool,
    pub local_flask_minion_heal_percent: f64,
    pub local_charm_duration_plus_percent: f64,
    pub local_maximum_prefixes_allowed: i64,
    pub local_maximum_suffixes_allowed: i64,
    pub local_item_benefit_socketable_as_if_helmet: bool,
    pub local_item_additional_skill_slots: u64,
    pub local_maximum_quality_is_allowed: u64,
    pub local_item_benefit_socketable_as_if_gloves: bool,
    pub local_item_benefit_socketable_as_if_boots: bool,
    pub local_charm_slots: u64,
    pub local_flask_use_on_affected_by_freeze: bool,
    pub local_flask_use_on_affected_by_bleed: bool,
    pub local_flask_use_on_affected_by_ignite: bool,
    pub local_flask_use_on_affected_by_poison: bool,
    pub local_flask_use_on_affected_by_shock: bool,
    pub local_flask_use_on_stunned: bool,
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
                "base_maximum_ward" => [Ward],
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
                "allies_in_presence_attack_speed_+%" => [AlliesInPresenceAttackSpeedIncreasePrecent],
                "allies_in_presence_accuracy_rating" => [AlliesInPresenceAccuracyRating],
                "local_accuracy_rating" => [AccuracyRating],
                "accuracy_rating" => [AccuracyRating],
                "accuracy_rating_+%" => [AccuracyRatingPercent],

                "chance_for_exerted_attacks_to_not_reduce_count_%" => [ChanceToNotConsumeExertedAttack],

                "spell_damage_+%" => [SpellDamageIncreasePrecent],
                "fire_damage_+%" => [FireDamageIncreasePrecent],
                "cold_damage_+%" => [ColdDamageIncreasePrecent],
                "lightning_damage_+%" => [LightningDamageIncreasePrecent],
                "chaos_damage_+%" => [ChaosDamageIncreasePrecent],

                "spell_physical_damage_+%" => [SpellPhysicalDamageIncreasePrecent],
                "trap_damage_+%" => [TrapDamageIncreasePrecent],
                "spell_skill_gem_level_+" => [SpellSkillGemLevelIncrease],
                "fire_spell_skill_gem_level_+" => [FireSpellSkillGemLevelIncrease],
                "cold_spell_skill_gem_level_+" => [ColdSpellSkillGemLevelIncrease],
                "lightning_spell_skill_gem_level_+" => [LightningSpellSkillGemLevelIncrease],
                "chaos_spell_skill_gem_level_+" => [ChaosSpellSkillGemLevelIncrease],
                "physical_spell_skill_gem_level_+" => [PhysicalSpellSkillGemLevelIncrease],
                "minion_skill_gem_level_+" => [MinionSkillGemLevelIncrease],
                "trap_skill_gem_level_+" => [TrapSkillGemLevelIncrease],
                "melee_skill_gem_level_+" => [MeleeSkillGemLevelIncrease],
                "projectile_skill_gem_level_+" => [ProjectileSkillGemLevelIncrease],

                "base_life_regeneration_rate_per_minute" => [LiferRegenPerMinute],
                "allies_in_presence_life_regeneration_rate_per_minute" => [AlliesInPresenceLifeRegenPerMinute],
                "mana_regeneration_rate_+%" => [ManaRegenerationRatePercent],
                "base_life_leech_from_physical_attack_damage_permyriad" => [BaseLifeLeechFromPhysicalAttackDamage],
                "base_mana_leech_from_physical_attack_damage_permyriad" => [BaseManaLeechFromPhysicalAttackDamage],

                "base_life_gained_on_enemy_death" => [BaseLifeGainedOnEnemyDeath],
                "base_mana_gained_on_enemy_death" => [BaseManaGainedOnEnemyDeath],
                "base_life_gain_per_target" => [BaseLifeGainedOnEnemyHit],
                "base_mana_gain_per_target" => [BaseManaGainedOnEnemyHit],

                "attack_speed_+%" => [AttackSpeedPercent],
                "base_cast_speed_+%" => [BaseCastSpeedIncreasePrecent],
                "allies_in_presence_cast_speed_+%" => [AlliesInPresenceCastSpeedIncreasePrecent],
                "trap_throwing_speed_+%" => [TrapThrowingSpeedIncreasePrecent],

                "critical_strike_chance_+%" => [CriticalStrikeChanceIncreasePrecent],
                "spell_critical_strike_chance_+%" => [SpellCriticalStrikeChanceIncreasePrecent],
                "attack_critical_strike_chance_+%" => [AttackCriticalStrikeChanceIncreasePrecent],
                "trap_critical_strike_chance_+%" => [TrapCriticalStrikeChanceIncreasePrecent],
                "allies_in_presence_critical_strike_chance_+%" => [AlliesInPresenceCriticalStrikeChanceIncreasePrecent],

                "base_critical_strike_multiplier_+" => [BaseCriticalStrikeMultiplierIncreasePrecent],
                "base_spell_critical_strike_multiplier_+" => [BaseSpellCriticalStrikeMultiplierIncreasePrecent],
                "attack_critical_strike_multiplier_+" => [AttackCriticalStrikeMultiplierIncrease],
                "trap_critical_strike_multiplier_+" => [TrapCriticalStrikeMultiplierIncrease],
                "allies_in_presence_critical_strike_multiplier_+" => [AlliesInPresenceCriticalStrikeMultiplierIncrease],

                "base_item_found_rarity_+%" => [BaseItemFoundRarityIncreasePrecent],

                "light_radius_+%" => [LightRadiusIncreasePrecent],
                "base_spirit_from_equipment" => [Spirit],

                "self_bleed_duration_+%" => [SelfBleedDurationDecreasePrecent],
                "self_poison_duration_+%" => [SelfPoisonDurationDecreasePrecent],
                "base_self_ignite_duration_-%" => [BaseIgniteDurationDecreasePrecent],
                "base_self_shock_duration_-%" => [BaseSelfShockDurationDecreasePrecent],
                "base_self_chill_duration_-%" => [BaseSelfChillDurationDecreasePrecent],
                "base_self_freeze_duration_-%" => [BaseSelfFreezeDurationDecreasePrecent],

                "base_self_critical_strike_multiplier_-%" => [ReduceCriticalStrikeMultiplierToSelf],
                "base_additional_physical_damage_reduction_%" => [PhysicalDamageReductionPrecent],
                "base_maximum_fire_damage_resistance_%" => [FireResistanceMax],
                "base_maximum_cold_damage_resistance_%" => [ColdResistanceMax],
                "base_maximum_lightning_damage_resistance_%" => [LightningResistanceMax],
                "base_maximum_chaos_damage_resistance_%" => [ChaosResistanceMax],
                "additional_maximum_all_elemental_resistances_%" => [AllElementalResistanceMax],

                "energy_shield_recharge_rate_+%" => [EnergyShieldRechargeRateIncreasePrecent],
                "energy_shield_delay_-%" => [EnergyShieldDelayDecreasePrecent],
                "armour_%_applies_to_fire_cold_lightning_damage" => [ArmourAppliesToElementalDamage],
                "base_armour_%_applies_to_chaos_damage" => [ArmourPercentAppliesToChaosDamage],
                "base_deflection_rating_%_of_armour" => [BaseDeflectionRatingPercentOfArmour],
                "base_deflection_rating_%_of_evasion_rating" => [EvasionAppliesToDeflection],
                "base_damage_%_deflected" => [DeflectDamageTaken],
                "evasion_rating_%_to_gain_as_armour" => [PercentEvasionRatingAsExtraArmour],
                "base_chance_to_pierce_%" => [BaseChanceToPierce],
                "base_number_of_crossbow_bolts" => [BaseNumberOfCrossbowBolts],
                "flask_life_recovery_rate_+%" => [FlaskLifeRecoveryRateIncreasePrecent],
                "flask_mana_recovery_rate_+%" => [FlaskManaRecoveryRateIncreasePrecent],
                "charm_duration_+%" => [CharmDurationIncreasePrecent],
                "flask_charges_gained_+%" => [FlaskChargesGainedIncreasePrecent],
                "flask_charges_used_+%" => [FlaskChargesReducedUsedIncreasePrecent],
                "charm_charges_gained_+%" => [CharmChargesGainedIncreasePrecent],
                "charm_charges_used_+%" => [CharmChargesReducedUsedIncreasePrecent],
                "ignite_chance_+%" => [IgniteChanceIncreasePrecent],
                "hit_damage_freeze_multiplier_+%" => [HitDamageFreezeMultiplierIncreasePrecent],
                "shock_chance_+%" => [ShockChanceIncreasePrecent],
                "base_projectile_speed_+%" => [BaseProjectileSpeedIncreasePrecent],
                "damage_taken_goes_to_life_over_4_seconds_%" => [DamageTakenGoesToLifeOver4Seconds],
                "damage_taken_goes_to_mana_%" => [DamageTakenGoesToMana],
                "elemental_damage_with_attack_skills_+%" => [ElementalDamageWithAttackSkillsIncreasePrecent],
                "non_skill_base_all_damage_%_to_gain_as_fire" => [NonSkillBaseAllDamageToGainAsFire],
                "non_skill_base_all_damage_%_to_gain_as_cold" => [NonSkillBaseAllDamageToGainAsCold],
                "non_skill_base_all_damage_%_to_gain_as_lightning" => [NonSkillBaseAllDamageToGainAsLightning],
                "non_skill_base_all_damage_%_to_gain_as_chaos" => [NonSkillBaseAllDamageToGainAsChaos],
                "damage_+%_with_bow_skills" => [DamageWithBowSkillsIncreasePrecent],
                "presence_area_+%" => [PresenceAreaIncreasePrecent],

                "minion_maximum_life_+%" => [MinionMaximumLifeIncreasePrecent],
                "trap_trigger_radius_+%" => [TrapTriggerRadiusIncreasePrecent],
                "charm_recover_X_life_when_used" => [CharmRecoverXLifeWhenUsed],
                "charm_recover_X_mana_when_used" => [CharmRecoverXManaWhenUsed],
                "charm_gain_X_guard_for_duration" => [CharmGainXGuardForDuration],

                "hit_damage_stun_multiplier_+%" => [HitDamageStunMultiplierIncreasePrecent],
                "chance_to_poison_on_hit_with_attacks_%" => [ChanceToPoisonOnHitWithAttacks],
                "bleed_on_hit_with_attacks_%" => [ChanceBleedOnHitWithAttacks],
                "base_arrow_speed_+%" => [BaseArrowSpeedIncreasePrecent],

                "additional_maximum_all_resistances_%" => [AllElementalResistanceMax],
                "fire_and_cold_damage_resistance_%" => [FireResistance, ColdResistance],
                "fire_and_lightning_damage_resistance_%" => [FireResistance, LightningResistance],
                "cold_and_lightning_damage_resistance_%" => [ColdResistance, LightningResistance],

                "fire_and_chaos_damage_resistance_%" => [FireResistance, ChaosResistance],
                "cold_and_chaos_damage_resistance_%" => [ColdResistance, ChaosResistance],
                "lightning_and_chaos_damage_resistance_%" => [LightningResistance, ChaosResistance],

                "flask_life_to_recover_+%" => [FlaskLifeRecoveryRateIncreasePrecent],
                "flask_mana_to_recover_+%" => [FlaskManaRecoveryRateIncreasePrecent],
                "stun_threshold_+%" => [StunThresholdPercent],
                "flask_recovery_amount_%_to_recover_instantly" => [FlaskRecoveryAmountPercentToRecoverInstantly],
                "generate_x_charges_for_any_flask_per_minute" => [GenerateXChargesForAnyFlaskPerMinute],
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
