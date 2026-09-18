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
    pub local_additional_block_chance_shield: f64,

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

    pub local_flat_chaos_min: u64,
    pub local_flat_chaos_max: u64,
    pub local_percent_chaos: f64,

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
    pub local_flat_critical_strike_chance: f64,
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
    pub local_reload_speed_plus_percent: f64,
    pub local_item_benefit_socketable_as_if_gloves: bool,
    pub local_item_benefit_socketable_as_if_boots: bool,
    pub local_charm_slots: u64,
    pub local_flask_use_on_affected_by_freeze: bool,
    pub local_flask_use_on_affected_by_bleed: bool,
    pub local_flask_use_on_affected_by_ignite: bool,
    pub local_flask_use_on_affected_by_poison: bool,
    pub local_flask_use_on_affected_by_shock: bool,
    pub local_flask_use_on_stunned: bool,
    pub local_flask_use_on_affected_by_slow: bool,
    pub local_flask_use_on_fire_damage_taken: bool,
    pub local_flask_use_on_cold_damage_taken: bool,
    pub local_flask_use_on_lightning_damage_taken: bool,
    pub local_flask_use_on_chaos_damage_taken: bool,
    pub local_flask_use_on_killing_rare_or_unique_enemy: bool,
    pub local_charm_trigger_when_cursed: bool,
    pub local_maximum_ward: u64,
    pub local_rage_on_hit: u64,
    pub local_weapon_accuracy_is_unaffected_by_distance: bool,
    pub local_culling_strike: bool,
    pub local_chance_to_bleed_on_hit: f64,
    pub local_cannot_be_thrown: bool,
    pub local_weapon_daze_chance: f64,
    pub local_always_hit: bool,
    pub local_explode_on_kill_with_crit: f64,
    pub local_crush_on_hit: bool,
    pub base_fire_damage: f64,
    pub local_display_grants_spear_throw_skill: bool,
    pub local_maim_on_hit: f64,
    pub local_projectile_speed: f64,
    pub local_weapon_range: f64,
    pub local_chance_to_blind_on_hit: f64,
    pub local_poison_on_hit: f64,
    pub local_apply_x_armour_break_on_crit: u64,
    pub local_weapon_roll_crits_twice: bool,
    pub local_attacks_cannot_be_blocked: bool,
    pub local_chain_chance: f64,
    pub local_crossbow_no_ammo_skills_and_give_alternate_grenade_default_attack: bool,
    pub local_jewel_effect_base_radius: i64,
    pub local_jewel_display_radius_change: i64,
    pub local_jewel_small_passive_in_radius_effect_plus_percent: f64,
    pub local_jewel_notable_passive_in_radius_effect_plus_percent: f64,
    pub local_jewel_transform_damage_increases_from_cold_lightning_to_fire: bool,
    pub local_jewel_transform_damage_increases_from_fire_lightning_to_cold: bool,
    pub local_jewel_transform_damage_increases_from_cold_fire_to_lightning: bool,
    pub local_jewel_copy_stats_from_unallocated_non_notable_passives_in_radius: bool,
    pub local_jewel_allocated_non_notable_passives_in_radius_grant_nothing: bool,
    pub local_non_unique_item_explicit_prefix_mod_magnitudes_plus_percent: f64,
    pub local_non_unique_item_explicit_suffix_mod_magnitudes_plus_percent: f64,
    pub local_chance_to_gain_onslaught_on_killing_blow: f64,
    pub local_chaos_penetration: f64,
    pub local_cold_penetration: f64,
    pub local_lightning_penetration: f64,
    pub local_fire_penetration: f64,
    pub local_display_fire_burst_on_hit: f64,
    pub poe1_local_display_grants_skill_bird_aspect_level: bool,
    pub poe1_local_display_grants_skill_cat_aspect_level: bool,
    pub poe1_local_display_grants_skill_crab_aspect_level: bool,
    pub poe1_local_display_grants_skill_spider_aspect_level: bool,
    pub local_explicit_elemental_damage_mod_effect_plus_percent: f64,
    pub local_force_corruption_outcome_two_enchants: bool,
    pub local_hand_wraps_energy_shield_per_level: u64,
    pub local_hand_wraps_evasion_rating_per_level: u64,
    pub local_socketed_items_effect_plus_percent: f64,
    pub local_ward_plus_percent: f64,
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
                "strength_+%" => [StrengthPercent],
                "dexterity_+%" => [DexterityPercent],
                "intelligence_+%" => [IntelligencePercent],
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

                "attack_damage_+%" => [AttackDamageIncreasePrecent],

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
                "physical_damage_+%" => [PhysicalDamageIncreasePrecent],

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
                "life_regeneration_rate_per_minute_%" => [LifeRegenPerMinutePercent],
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
                "base_spirit" => [Spirit],
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
                "flask_duration_+%" => [FlaskDurationIncreasePrecent],
                "flask_charges_used_+%" => [FlaskChargesReducedUsedIncreasePrecent],
                "charm_charges_gained_+%" => [CharmChargesGainedIncreasePrecent],
                "charm_charges_used_+%" => [CharmChargesReducedUsedIncreasePrecent],
                "ignite_chance_+%" => [IgniteChanceIncreasePrecent],
                "damage_+%_while_using_charm" => [DamageWhileUsingCharmIncreasePrecent],
                "hit_damage_freeze_multiplier_+%" => [HitDamageFreezeMultiplierIncreasePrecent],
                "shock_chance_+%" => [ShockChanceIncreasePrecent],
                "shock_duration_+%" => [ShockDurationIncreasePrecent],
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
                "minion_additional_physical_damage_reduction_%" => [MinionAdditionalPhysicalDamageReductionIncreasePrecent],
                "minion_elemental_resistance_%" => [MinionFireResistance,MinionColdResistance,MinionLightningResistance,MinionChaosResistance],
                "minion_resummon_speed_+%" => [MinionResummonSpeedIncreasePrecent],
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
                "life_flask_charges_gained_+%" => [LifeFlaskChargesGainedIncreasePrecent],
                "flask_mana_to_recover_+%" => [FlaskManaRecoveryRateIncreasePrecent],
                "mana_flask_charges_gained_+%" => [ManaFlaskChargesGainedIncreasePrecent],
                "stun_threshold_+%" => [StunThresholdPercent],
                "flask_recovery_amount_%_to_recover_instantly" => [FlaskRecoveryAmountPercentToRecoverInstantly],
                "generate_x_charges_for_any_flask_per_minute" => [GenerateXChargesForAnyFlaskPerMinute],
                "maximum_mana_%_gained_on_kill" => [MaximumManaGainedOnKillPercent],
                "base_mana_leech_amount_+%" => [BaseManaLeechAmountIncreasePrecent],
                "ailment_threshold_+%" => [AilmentThresholdPercent],
                "ailment_threshold_+" => [AilmentThresholdFlat],
                "base_slow_potency_+%" => [BaseSlowPotencyPercent],
                "spear_attack_speed_+%" => [SpearAttackSpeedIncreasePrecent],
                "spear_critical_strike_multiplier_+" => [SpearCriticalStrikeMultiplierIncreasePrecent],
                "spear_damage_+%" => [SpearDamageIncreasePrecent],
                "stun_threshold_+_from_%_maximum_energy_shield" => [StunThresholdFromMaximumEnergyShieldPercent],
                "ailment_threshold_+_from_%_maximum_energy_shield" => [AilmentThresholdFromMaximumEnergyShieldPercent],
                "stun_threshold_+%_when_not_stunned_recently" => [StunThresholdWhenNotStunnedRecentlyPercent],
                "sword_damage_+%" => [SwordDamageIncreasePrecent],
                "thorns_damage_+%" => [ThornsDamageIncreasePrecent],
                "totem_damage_+%" => [TotemDamageIncreasePrecent],
                "trap_damage_+%" => [TrapDamageIncreasePrecent],
                "summon_totem_cast_speed_+%" => [SummonTotemCastSpeedIncreasePrecent],
                "totem_life_+%" => [TotemLifeIncreasePrecent],
                "sword_attack_speed_+%" => [SwordAttackSpeedIncreasePrecent],
                "base_bleeding_effect_+%" => [BaseBleedingEffectPercent],
                "movement_speed_penalty_+%_while_performing_action" => [MovementSpeedPenaltyReductionPercent],
                "base_damage_removed_from_mana_before_life_%" => [BaseDamageRemovedFromManaBeforeLifePercent],
                "self_elemental_status_duration_-%" => [SelfStatusAilmentDurationReductionPercent],
                "corrupted_skill_gem_level_+" => [CorruptedSkillGemLevelIncrease],
                "all_skill_gem_level_+" => [AllSkillGemLevelIncrease],
                "ward_regeneration_rate_+%" => [WardRegenerationRateIncreasePrecent],

                "base_ignite_effect_+%" => [BaseIgniteIncereasedMagnitudePrecent],
                "base_damage_taken_+%" => [BaseDamageTakenIncreasePrecent],

                "shield_armour_evasion_energy_shield_+%" => [IncreaseEvasionPercentArmourPercentEnergyShieldPercentFromShield],
                "melee_splash" => [StrikesDealMeleeSplash],
                "warcry_empowers_next_x_melee_attacks" => [WarCryEmpowersNextXMeleeAttacks],
                "minion_damage_+%" => [MinionDamageIncreasePrecent],
                "gain_x_rage_on_melee_hit" => [GainXRageOnMeleeHit],
                "gain_x_rage_when_hit" => [GainXRageWhenGetByEnemyHit],
                "shock_effect_+%" => [ShockEffectIncreasePrecent],
                "maximum_rage" => [MaximumRage],
                "minion_accuracy_rating_+%" => [MinionAccuracyRatingIncreasePrecent],
                "minion_skill_area_of_effect_+%" => [MinionSkillAreaOfEffectIncreasePrecent],
                "minion_chaos_resistance_%" => [MinionChaosResistanceIncreasePrecent],
                "minion_critical_strike_chance_+%" => [MinionCriticalStrikeChanceIncreasePrecent],
                "minion_critical_strike_multiplier_+" => [MinionCriticalStrikeMultiplierIncreasePrecent],
                "minion_attack_and_cast_speed_+%" => [MinionAttackAndCastSpeedIncreasePrecent],
                "melee_damage_+%" => [MeleeDamageIncreasePrecent],
                "mark_effect_+%" => [MarkEffectIncreasePrecent],
                "mark_skill_duration_+%" => [MarkSkillDurationIncreasePrecent],
                "mark_use_speed_+%" => [MarkUseSpeedIncreasePrecent],
                "additional_block_%" => [AdditionalBlockIncreasePrecent],
                "faster_bleed_%" => [FasterBleedDamage],
                "spells_cost_life_instead_of_mana_%" => [SpellsCostLifeInsteadOfMana],
                "chance_to_fire_1_additional_projectile_%_with_rollover_with_bow_attacks" => [ChanceToFire1AdditionalProjectileWithRolloverWithBowAttacks],
                "projectile_attack_range_+%" => [ProjectileAttackRangePrecent],
                "projectile_speed_+%_with_crossbow_skills" => [ProjectileSpeedIncreasePrecentForCrossbowSkills],
                "grenade_skill_number_of_additional_projectiles" => [GrenadeSkillNumberOfAdditionalProjectiles],
                "additional_ballista_totems_allowed" => [AdditionalBallistaTotemsAllowed],
                "grenade_skill_cooldown_count_+" => [GrenadeSkillCooldownCount],
                "grenade_skill_cooldown_speed_+%" => [GrenadeSkillCooldownSpeedIncreasePrecent],
                "placing_traps_cooldown_recovery_+%" => [PlacingTrapsCooldownRecoveryForThrowingIncreasePrecent],
                "ailment_chance_+%" => [AilmentChanceIncreasePrecent],
                "ailment_effect_+%" => [AilmentEffectIncreasePrecent],
                "base_skill_area_of_effect_+%" => [BaseSkillAreaOfEffectIncreasePrecent],
                "armour_break_amount_+%" => [ArmourBreakAmountIncreasePrecent],
                "armour_break_and_sundered_armour_duration_+%" => [ArmourBreakDurationIncreasePrecent],
                "aura_effect_+%" => [AuraEffectIncreasePrecent],
                "axe_damage_+%" => [AxeDamageIncreasePrecent],
                "axe_attack_speed_+%" => [AxeAttackSpeedIncreasePrecent],

                "base_chance_to_inflict_bleeding_%" => [BaseChanceToInflictBleeding],
                "base_bleed_duration_+%" => [BaseBleedDurationIncreasePrecent],
                "blind_effect_+%" => [BlindEffectIncreasePrecent],
                "attacks_chance_to_blind_on_hit_%" => [ChanceToBlindOnHitWithAttacks],
                "block_chance_+%" => [BlockChanceIncreasePrecent],
                "damage_+%_to_rare_and_unique_enemies" => [DamageToRareAndUniqueEnemiesIncreasePrecent],
                "bow_accuracy_rating_+%" => [AccuracyRatingIncreaseWithBowPrecent],
                "bow_damage_+%" => [BowDamageIncreasePrecent],
                "bow_attack_speed_+%" => [BowAttackSpeedIncreasePrecent],
                "projectile_chance_to_chain_1_extra_time_from_terrain_%" => [ProjectileChanceToChain1ExtraTimeFromTerrainIncreasePrecent],
                "chill_duration_+%" => [ChillDurationIncreasePrecent],
                "base_reduce_enemy_cold_resistance_%" => [ColdResistancePenetration],
                "base_cooldown_speed_+%" => [BaseCooldownSpeedIncreasePrecent],
                "damage_+%_if_you_have_consumed_a_corpse_recently" => [DamageIfYouHaveConsumedACorpseRecentlyIncreasePrecent],
                "critical_hit_damaging_ailment_effect_+%" => [CriticalHitDamagingAilmentEffectIncreasePrecent],
                "crossbow_damage_+%" => [CrossbowDamageIncreasePrecent],
                "reload_speed_+%" => [ReloadSpeedIncreasePrecent],
                "%_chance_for_crossbow_reload_to_be_instant" => [CrossbowReloadInstantChanceIncreasePrecent],
                "crossbow_attack_speed_+%" => [CrossbowAttackSpeedIncreasePrecent],
                "curse_area_of_effect_+%" => [CurseAreaOfEffectIncreasePrecent],
                "curse_delay_+%" => [CurseDelayIncreasePrecent],
                "base_curse_duration_+%" => [BaseCurseDurationIncreasePrecent],
                "curse_effect_+%" => [CurseEffectIncreasePrecent],
                "dagger_critical_strike_chance_+%" => [DaggerCriticalStrikeChanceIncreasePrecent],
                "dagger_damage_+%" => [DaggerDamageIncreasePrecent],
                "dagger_attack_speed_+%" => [DaggerAttackSpeedIncreasePrecent],
                "damage_+%_against_enemies_with_fully_broken_armour" => [DamageAgainstEnemiesWithFullyBrokenArmourIncreasePrecent],
                "damaging_ailment_duration_+%" => [DamagingAilmentDurationIncreasePrecent],
                "base_chance_to_daze_%" => [BaseChanceToDazeIncreasePrecent],
                "debuff_time_passed_+%" => [DebuffTimePassedIncreasePrecent],
                "ignite_shock_chill_duration_+%" => [IgniteShockChillDurationIncreasePrecent],
                "elemental_damage_+%" => [ElementalDamageIncreasePrecent],
                "empowered_attack_damage_+%" => [EmpoweredAttackDamageIncreasePrecent],
                "energy_generated_+%" => [EnergyGeneratedIncreasePrecent],
                "damaging_ailments_deal_damage_+%_faster" => [DamagingAilmentsDealDamageIncreasePrecent],
                "base_reduce_enemy_fire_resistance_%" => [FireResistancePenetration],
                "flail_critical_strike_chance_+%" => [FlailCriticalStrikeChanceIncreasePrecent],
                "flail_damage_+%" => [FlailDamageIncreasePrecent],
                "energy_shield_from_focus_+%" => [EnergyShieldFromFocusIncreasePrecent],
                "chance_to_fork_extra_projectile_%" => [ChanceToForkExtraProjectileIncreasePrecent],
                "freeze_threshold_+%" => [FreezeThresholdIncreasePrecent],
                "damage_+%_with_herald_skills" => [DamageWithHeraldSkillsIncreasePrecent],
                "skill_effect_duration_+%" => [SkillEffectDurationIncreasePrecent],
                "knockback_distance_+%" => [KnockbackDistanceIncreasePrecent],
                "base_skill_cost_life_instead_of_mana_%" => [BaseSkillCostLifeInsteadOfManaIncreasePrecent],
                "base_life_leech_amount_+%" => [BaseLifeLeechAmountIncreasePrecent],
                "recover_%_maximum_life_on_kill" => [RecoverMaximumLifeOnKillIncreasePrecent],
                "damage_taken_goes_to_life_over_4_seconds_%" => [DamageTakenGoesToLifeOver4SecondsIncreasePrecent],
                "life_regeneration_rate_+%" => [LifeRegenerationRateIncreasePrecent],
                "base_reduce_enemy_lightning_resistance_%" => [LightningResistancePenetration],
                "mace_damage_+%" => [MaceDamageIncreasePrecent],
                "mace_hit_damage_stun_multiplier_+%" => [MaceHitDamageStunMultiplierIncreasePrecent],
                "offering_duration_+%" => [OfferingDurationIncreasePrecent],
                "offering_life_+%" => [OfferingLifeIncreasePrecent],
                "hit_damage_pin_multiplier_+%" => [HitDamagePinMultiplierIncreasePrecent],
                "base_chance_to_poison_on_hit_%" => [BaseChanceToPoisonOnHitIncreasePrecent],
                "base_poison_effect_+%" => [BasePoisonEffectIncreasePrecent],
                "base_poison_duration_+%" => [BasePoisonDurationIncreasePrecent],
                "projectile_damage_+%" => [ProjectileDamageIncreasePrecent],
                "quarterstaff_damage_+%" => [QuarterstaffDamageIncreasePrecent],
                "quarterstaff_hit_damage_freeze_multiplier_+%" => [QuarterstaffHitDamageFreezeMultiplierIncreasePrecent],
                "quarterstaff_attack_speed_+%" => [QuarterstaffAttackSpeedIncreasePrecent],
                "quiver_mod_effect_+%" => [QuiverModEffectIncreasePrecent],
                "triggered_spell_spell_damage_+%" => [TriggeredSpellSpellDamageIncreasePrecent],
                "unarmed_damage_+%" => [UnarmedDamageIncreasePrecent],
                "warcry_buff_effect_+%" => [WarcryBuffEffectIncreasePrecent],
                "warcry_cooldown_speed_+%" => [WarcryCooldownSpeedIncreasePrecent],
                "warcry_damage_+%" => [WarcryDamageIncreasePrecent],
                "warcry_speed_+%" => [WarcrySpeedIncreasePrecent],
                "weapon_swap_speed_+%" => [WeaponSwapSpeedIncreasePrecent],
                "withered_magnitude_+%" => [WitheredMagnitudeIncreasePrecent],
                "unarmed_attack_speed_+%" => [UnarmedAttackSpeedIncreasePrecent],
                "projectile_damage_+%_if_youve_dealt_melee_hit_recently" => [ProjectileDamageIncreasePrecentIfYouveDealtMeleeHitRecently],
                "melee_damage_+%_if_youve_dealt_projectile_attack_hit_recently" => [MeleeDamageIncreasePrecentIfYouveDealtProjectileAttackHitRecently],
                "parry_damage_+%" => [ParryDamageIncreasePrecent],
                "parry_skill_effect_duration_+%" => [ParrySkillEffectDurationIncreasePrecent],
                "parry_stun_threshold_+%_during_parry" => [ParryStunThresholdIncreasePrecentDuringParry],
                "volatility_on_kill_%_chance" => [VolatilityOnKillChanceIncreasePrecent],
                "companion_damage_+%" => [CompanionDamageIncreasePrecent],
                "companion_maximum_life_+%" => [CompanionMaximumLifeIncreasePrecent],
                "hazard_damage_+%" => [HazardDamageIncreasePrecent],
                "chance_to_inflict_incision_on_attack_hit_%" => [ChanceToInflictIncisionOnAttackHitIncreasePrecent],
                "glory_generation_+%_for_banners" => [GloryGenerationIncreasePrecentForBanners],
                "banner_area_of_effect_+%" => [BannerAreaOfEffectIncreasePrecent],
                "banner_duration_+%" => [BannerDurationIncreasePrecent],
                "apply_debilitate_on_hit_while_emerald_sapphire_socketed" => [ApplyDebilitateOnHitWhileEmeraldSapphireSocketed],
                "apply_blind_on_hit_while_ruby_sapphire_socketed" => [ApplyBlindOnHitWhileRubySapphireSocketed],
                "apply_exposure_on_hit_while_ruby_emerald_socketed" => [ApplyExposureOnHitWhileRubyEmeraldSocketed],
                "cannot_die" => [CannotDie],
                "gold_+%_from_enemies" => [GoldFromEnemiesIncreasePrecent],
                "max_endurance_charges" => [MaxEnduranceCharges],
                "max_frenzy_charges" => [MaxFrenzyCharges],
                "max_power_charges" => [MaxPowerCharges],
                "additional_maximum_block_%" => [AdditionalMaximumBlockIncreasePrecent],
                "life_gained_on_block" => [LifeGainedOnBlock],
                "mana_gained_on_block" => [ManaGainedOnBlock],
                "damage_+%" => [DamageIncreasePrecent],
                "skill_speed_%" => [SkillSpeedIncreasePrecent],
                "base_debuff_slow_magnitude_+%" => [BaseDebuffSlowMagnitudeIncreasePrecent],
                "generate_x_charges_for_life_flasks_per_minute" => [GenerateXChargesForLifeFlasksPerMinute],
                "generate_x_charges_for_mana_flasks_per_minute" => [GenerateXChargesForManaFlasksPerMinute],
                "generate_x_charges_for_charms_per_minute" => [GenerateXChargesForCharmsPerMinute],
                "abyssal_wasting_effect_+%" => [AbyssalWastingEffectIncreasePrecent],
                "add_power_charge_on_critical_strike_%" => [AddPowerChargeOnCriticalStrikeIncreasePrecent],
                "additional_base_critical_strike_chance" => [AdditionalBaseCriticalStrikeChance],
                "additional_block_chance_against_projectiles_%" => [AdditionalBlockChanceAgainstProjectilesIncreasePrecent],
                "additional_combo_gain_chance_%" => [AdditionalComboGainChanceIncreasePrecent],
                "additional_dexterity_and_intelligence" => [Dexterity, Intelligence],
                "additional_infusion_gain_chance_%" => [AdditionalInfusionGainChanceIncreasePrecent],
                "additional_maximum_infusion_stacks" => [AdditionalMaximumInfusionStacks],
                "additional_physical_damage_reduction_%_during_flask_effect" => [AdditionalPhysicalDamageReductionIncreasePrecent],
                "aggravate_bleeding_older_than_ms_on_hit" => [AggravateBleedingOlderThanMsOnHit],
                "all_skill_gem_quality_+" => [GlobalSkillGemQualityPluss],
                "allies_in_presence_resist_all_elements_%" => [AlliesInPresenceResistAllElementsIncreasePrecent],
                "apply_anaemia_magnitude_on_hit" => [ApplyAnaemiaMagnitudeOnHit],
                "arcane_surge_effect_on_self_+%" => [ArcaneSurgeEffectOnSelfIncreasePrecent],
                "archon_delay_expires_x%_faster" => [ArchonDelayExpiresXPercentFaster],
                "archon_duration_+%" => [ArchonDurationIncreasePrecent],
                "archon_effect_on_you_+%" => [ArchonEffectOnYouIncreasePrecent],
                "armour_break_equal_to_%_physical_damage_dealt_on_critical_strike_with_spells" => [ArmourBreakEqualToPhysicalDamageDealtOnCriticalStrikeWithSpells],
                "armour_break_physical_damage_%_dealt_as_armour_break" => [ArmourBreakPhysicalDamageIncreasePrecent],
                "arrow_speed_additive_modifiers_also_apply_to_bow_damage" => [ArrowSpeedAdditiveModifiersAlsoApplyToBowDamage],
                "attack_added_chaos_damage_%_of_maximum_life" => [AttackAddedChaosDamageIncreasePrecent],
                "attack_added_physical_damage_%_of_maximum_life" => [AttackAddedPhysicalDamageIncreasePrecent],
                "attack_added_cold_damage_%_of_maximum_life" => [AttackAddedColdDamageIncreasePrecent],
                "attack_added_fire_damage_%_of_maximum_life" => [AttackAddedFireDamageIncreasePrecent],
                "attack_added_lightning_damage_%_of_maximum_life" => [AttackAddedLightningDamageIncreasePrecent],
                "attack_and_cast_speed_+%" => [AttackAndCastSpeedIncreasePrecent],
                "attack_and_cast_speed_+%_during_flask_effect" => [AttackAndCastSpeedIncreasePrecentDuringFlaskEffect],
                "attack_area_of_effect_+%" => [AttackAreaOfEffectIncreasePrecent],
                "attack_area_of_effect_+%_per_10_int" => [AttackAreaOfEffectIncreasePrecentPer10Int],
                "attack_damage_+%_when_on_low_life" => [AttackDamageIncreasePrecentWhenOnLowLife],
                "attack_mana_cost_+%" => [ReducedAttackManaCostPrecent],
                "attack_maximum_added_chaos_damage" => [AttackMaximumAddedChaosDamage],
                "attack_minimum_added_chaos_damage" => [AttackMinimumAddedChaosDamage],
                "attack_skill_gem_level_+" => [AttackSkillGemLevelIncrease],
                "attack_skills_have_added_lightning_damage_equal_to_%_of_maximum_mana" => [AttackSkillsHaveAddedLightningDamageEqualToPrecentOfMaximumMana],
                "attack_speed_+%_if_not_been_hit_recently" => [AttackSpeedIncreasePrecentIfNotBeenHitRecently],
                "attack_speed_+%_when_on_full_life" => [AttackSpeedIncreasePrecentWhenOnFullLife],
                "attack_speed_+%_while_in_presence_of_companion" => [AttackSpeedIncreasePrecentWhileInPresenceOfCompanion],
                "attack_speed_+%_while_missing_ward" => [AttackSpeedIncreasePrecentWhileMissingWard],
                "attacks_num_of_additional_chains" => [AttackNumberOfAdditionalChains],
                "avoid_all_elemental_status_%" => [AvoidAllElementalStatusPrecent],
                "base_all_ailment_duration_+%" => [BaseAllAilmentDurationPrecent],
                "base_all_ailment_duration_on_self_+%" => [ReduceBaseAllAilmentDurationOnSelfPrecent],
                "base_all_attributes" => [Strength, Dexterity, Intelligence],
                "base_attack_skill_cost_efficiency_+%" => [BaseAttackSkillCostEfficiencyPrecent],
                "base_avoid_chill_%" => [BaseAvoidChillPrecent],
                "base_avoid_freeze_%" => [BaseAvoidFreezePrecent],
                "base_avoid_ignite_%" => [BaseAvoidIgnitePrecent],
                "base_avoid_shock_%" => [BaseAvoidShockPrecent],
                "base_avoid_stun_%" => [BaseAvoidStunPrecent],
                "base_bleed_chance_is_poison_chance_instead" => [BaseBleedChanceIsPoisonChanceInstead],
                "base_chance_to_shock_%" => [BaseChanceToShockPrecent],
                "base_damaging_ailment_effect_+%" => [BaseDamagingAilmentEffectPrecent],
                "base_elemental_status_ailment_duration_+%" => [BaseElementalStatusAilmentDurationPrecent],
                "base_enemy_critical_strike_chance_+%_against_self" => [BaseEnemyCriticalStrikeChancePrecent],
                "base_life_cost_efficiency_+%" => [BaseLifeCostEfficiencyPrecent],
                "base_life_leech_does_not_stop_at_full_life" => [BaseLifeLeechDoesNotStopAtFullLife],
                "base_life_leech_rate_+%" => [BaseLifeLeechRatePrecent],
                "base_mana_cost_+_with_non_channelling_skills" => [BaseManaCostPrecent],
                "base_mana_cost_efficiency_+%" => [BaseManaCostEfficiencyPrecent],
                "base_mana_leech_rate_+%" => [BaseManaLeechRatePrecent],
                "base_mana_reservation_efficiency_+%" => [BaseManaReservationEfficiencyPrecent],
                "base_maximum_seals_for_skill" => [BaseMaximumSealsForSkill],
                "base_minion_duration_+%" => [BaseMinionDurationPrecent],
                "base_number_of_essence_spirits_allowed" => [BaseNumberOfEssenceSpiritsAllowed],
                "base_onlsaught_on_hit_%_chance" => [BaseOnlsaughtOnHitPrecent],
                "base_poison_chance_is_bleed_chance_instead" => [BasePoisonChanceIsBleedChanceInstead],
                "base_rage_cost_efficiency_+%" => [BaseRageCostEfficiencyPrecent],
                "base_reservation_efficiency_+%" => [BaseReservationEfficiencyPrecent],
                "base_skill_cost_efficiency_+%" => [BaseSkillCostEfficiencyPrecent],
                "base_spell_cooldown_speed_+%" => [BaseSpellCooldownSpeedPrecent],
                "base_spell_mana_cost_efficiency_+%" => [BaseSpellManaCostEfficiencyPrecent],
                "base_spirit_reservation_efficiency_+%" => [BaseSpiritReservationEfficiencyPrecent],
                "base_thorns_critical_strike_chance" => [BaseThornsCriticalStrikeChance],
                "base_unholy_might_granted_magnitude_+%" => [BaseUnholyMightGrantedMagnitudePrecent],
                "bell_hit_limit" => [BellHitLimit],
                "bleed_chance_+%" => [BleedChancePrecent],
                "block_%_damage_taken_while_active_blocking" => [BlockDamageTakenWhileActiveBlockingPrecent],
                "body_armour_+%" => [BodyArmourPrecent],
                "body_armour_evasion_rating_+%" => [BodyArmourEvasionRatingPrecent],
                "broken_armour_and_sundered_armour_debuff_effect_+%" => [BrokenArmourAndSunderedArmourDebuffEffectPrecent],
                "cannot_be_inflicted_by_corrupted_blood" => [CannotBeInflictedByCorruptedBlood],
                "cannot_be_poisoned" => [CannotBePoisoned],
                "cast_speed_+%_during_mana_flask_effect" => [CastSpeedPrecentDuringManaFlaskEffect],
                "cast_speed_+%_per_num_unique_spells_cast_recently" => [CastSpeedPrecentPerNumUniqueSpellsCastRecently],
                "cast_speed_+%_when_on_full_life" => [CastSpeedPrecentWhenOnFullLife],
                "cast_speed_+%_when_on_low_life" => [CastSpeedPrecentWhenOnLowLife],
                "cast_speed_+%_while_on_full_mana" => [CastSpeedPrecentWhileOnFullMana],
                "chance_%_to_gain_archon_of_nature_on_overgrowing_plant" => [ChanceToGainArchonOfNatureOnOvergrowingPlantPrecent],
                "chance_%_to_gain_archon_of_undeath_when_you_create_an_offering" => [ChanceToGainArchonOfUndeathWhenYouCreateAnOfferingPrecent],
                "chance_to_crush_on_hit_%" => [ChanceToCrushOnHitPrecent],
                "chance_to_fire_1_additional_projectile_%_with_rollover" => [ChanceToFire1AdditionalProjectileWithRolloverPrecent],
                "chance_to_gain_power_charge_on_stun_%" => [ChanceToGainPowerChargeOnStunPrecent],
                "chance_to_not_consume_infusion_%_if_lost_archon_in_past_6_seconds" => [ChanceToNotConsumeInfusionPrecentIfLostArchonInPast6Seconds],
                "chance_to_retain_40%_of_glory_on_use_%" => [ChanceToRetain40PrecentOfGloryOnUsePrecent],
                "chaos_damage_resistance_%_per_poison_stack" => [ChaosDamageResistancePrecentPerPoisonStack],
                "chaos_damage_taken_over_time_+%" => [ChaosDamageTakenOverTimePrecent],
                "chaos_dot_multiplier_+" => [ChaosDotMultiplierPrecent],
                "chaos_resistance_+_while_using_flask" => [ChaosResistancePrecentWhileUsingFlask],
                "charge_skip_consume_chance_%" => [ChargeSkipConsumeChancePrecent],
                "charms_%_chance_to_not_consume_charges" => [CharmsPrecentChanceToNotConsumeCharges],
                "cold_damage_+%_cold_infusion_collected_last_8_seconds" => [ColdDamagePrecentColdInfusionCollectedLast8Seconds],
                "cold_damage_taken_goes_to_life_over_4_seconds_%" => [ColdDamageTakenPrecentGoesToLifeOver4Seconds],
                "cold_dot_multiplier_+" => [ColdDotMultiplierPrecent],
                "companion_attack_speed_+%" => [CompanionAttackSpeedPrecent],
                "created_remnants_have_%_chance_to_duplicate_pick_up_results" => [CreatedRemnantsPrecentChanceToDuplicatePickUpResults],
                "critical_hit_damage_bonus_+%_vs_enemies_within_2m_distance" => [CriticalHitDamageBonusPrecentVsEnemiesWithin2mDistance],
                "critical_strike_chance_+%_vs_enemies_further_than_6m_distance" => [CriticalStrikeChancePrecentVsEnemiesFurtherThan6mDistance],
                "critical_strike_chance_+%_vs_marked_enemies" => [CriticalStrikeChancePrecentVsMarkedEnemies],
                "curse_effect_on_self_+%" => [CurseEffectOnSelfPrecent],
                "damage_+%_against_enemies_marked_by_you" => [DamagePrecentAgainstEnemiesMarkedByYou],
                "damage_+%_per_poison_up_to_75%" => [DamagePrecentPerPoisonUpTo75],
                "damage_+%_vs_immobilised_enemies" => [DamagePrecentVsImmobilisedEnemies],
                "damage_+%_while_in_presence_of_companion" => [DamagePrecentWhileInPresenceOfCompanion],
                "damage_+%_while_leeching" => [DamagePrecentWhileLeeching],
                "damage_+%_while_totem_active" => [DamagePrecentWhileTotemActive],
                "damage_removed_from_spectres_before_life_or_es_%" => [DamageRemovedFromSpectresBeforeLifeOrEsPrecent],
                "damage_taken_goes_to_life_mana_es_over_4_seconds_%" => [DamageTakenGoesToLifeManaEsOver4SecondsPrecent],
                "damage_vs_cursed_enemies_per_enemy_curse_+%" => [DamageVsCursedEnemiesPerEnemyCursePrecent],
                "deal_1000_chaos_damage_per_second_for_10_seconds_on_hit" => [Deal1000ChaosDamagePerSecondFor10SecondsOnHit],
                "deflected_hit_damage_taken_%_recouped_as_life" => [DeflectedHitDamageTakenPrecentRecoupedAsLife],
                "deflection_rating_+%" => [DeflectionRatingPrecent],
                "dexterity_and_intelligence_+%" => [DexterityAndIntelligencePrecent],
                "dodge_roll_base_travel_distance" => [DodgeRollBaseTravelDistance],
                "dummy_stat_display_nothing" => [DummyStatDisplayNothing],
                "elemental_ailment_types_apply_damage_taken_+%" => [ElementalAilmentTypesApplyDamageTakenPrecent],
                "elemental_penetration_%_during_flask_effect" => [ElementalPenetrationPrecentDuringFlaskEffect],
                "elemental_skill_limit_+" => [ElementalSkillLimitPrecent],
                "enemies_chaos_resistance_%_while_cursed" => [EnemiesChaosResistancePrecentWhileCursed],
                "enemies_explode_on_kill" => [EnemiesChanceExplodeOnKill],
                "energy_shield_recharge_rate_+%_if_blocked_recently" => [EnergyShieldRechargeRatePrecentIfBlockedRecently],
                "essence_abyss_guaranteed_pick" => [EssenceAbyssGuaranteedPick],
                "essence_buff_elemental_damage_taken_+%" => [EssenceBuffElementalDamageTakenPrecent],
                "essence_display_elemental_damage_taken_while_not_moving_+%" => [EssenceDisplayElementalDamageTakenWhileNotMovingPrecent],
                "evasion_rating_+%" => [EvasionRatingPrecent],
                "evasion_rating_+%_while_leeching" => [EvasionRatingPrecentWhileLeeching],
                "explode_burning_enemies_for_10%_life_as_fire_on_kill_chance_%" => [ExplodeBurningEnemiesFor10LifeAsFireOnKillChancePrecent],
                "explode_enemies_for_10%_life_as_physical_on_kill_chance_%" => [ExplodeEnemiesFor10LifeAsPhysicalOnKillChancePrecent],
                "explode_enemies_for_25%_life_as_chaos_on_kill_chance_%" => [ExplodeEnemiesFor25LifeAsChaosOnKillChancePrecent],
                "exposure_effect_+%" => [ExposureEffectPrecent],
                "fire_damage_+%_if_fire_infusion_collected_last_8_seconds" => [FireDamagePrecentIfFireInfusionCollectedLast8Seconds],
                "fire_damage_taken_goes_to_life_over_4_seconds_%" => [FireDamageTakenGoesToLifeOver4SecondsPrecent],
                "fire_dot_multiplier_+" => [FireDotMultiplierPrecent],
                "fire_spell_additional_critical_strike_chance_permyriad" => [FireSpellAdditionalCriticalStrikeChancePerMyriad],
                "fish_quantity_+%" => [FishQuantityPrecent],
                "fish_rarity_+%" => [FishRarityPrecent],
                "fishing_hook_type" => [FishingHookType],
                "fishing_line_strength_+%" => [FishingLineStrengthPrecent],
                "fishing_lure_type" => [FishingLureType],
                "fishing_pool_consumption_+%" => [FishingPoolConsumptionPrecent],
                "fishing_range_+%" => [FishingRangePrecent],
                "flasks_%_chance_to_not_consume_charges" => [FlasksPrecentChanceToNotConsumeCharges],
                "fully_break_enemies_armour_on_heavy_stun_with_shield_skills" => [FullyBreakEnemiesArmourOnHeavyStunWithShieldSkills],
                "gain_alchemists_genius_on_flask_use_%" => [GainAlchemistsGeniusOnFlaskUsePrecent],
                "gain_arcane_surge_on_crit_%_chance" => [GainArcaneSurgeOnCritPrecent],
                "gain_onslaught_for_3_seconds_%_chance_when_hit" => [GainOnslaughtFor3SecondsPrecent],
                "global_armour_evasion_energy_shield_+%" => [GlobalArmourEvasionEnergyShieldPrecent],
                "global_item_attribute_requirements_+%" => [GlobalItemAttributeReducedRequirementsPrecent],
                "glory_generation_+%" => [GloryGenerationPrecent],
                "grenade_skill_%_chance_to_explode_twice" => [GrenadeSkillPrecentChanceToExplodeTwice],
                "grenade_skill_damage_+%" => [GrenadeSkillDamagePrecent],
                "grenade_skill_duration_+%" => [GrenadeSkillDurationPrecent],
                "hand_wraps_attack_damage_+%_final_on_low_mana" => [HandWrapsAttackDamagePrecentFinalOnLowMana],
                "hand_wraps_damage_taken_+%_final_on_low_life" => [HandWrapsDamageTakenPrecentFinalOnLowLife],
                "hand_wraps_evasion_rating_and_energy_shield_+%_final" => [HandWrapsMoreGlobalEvasionEnergyShield],
                "has_grave_command" => [GrantsGraveCommandSkill],
                "have_unholy_might" => [HaveUnholyMight],
                "heavy_stun_poise_decay_rate_+%" => [HeavyStunPoiseDecayRatePrecent],
                "hinder_enemy_chaos_damage_taken_+%" => [HinderEnemyChaosDamageTakenPrecent],
                "hinder_enemy_elemental_damage_taken_+%" => [HinderEnemyElementalDamageTakenPrecent],
                "hinder_enemy_physical_damage_taken_+%" => [HinderEnemyPhysicalDamageTakenPrecent],
                "hit_damage_freeze_multiplier_+%_if_consumed_power_charge_recently" => [HitDamageFreezeMultiplierPrecentIfConsumedPowerChargeRecently],
                "hit_damage_immobilisation_multiplier_+%" => [HitDamageImmobilisationMultiplierPrecent],
                "ignite_effect_+%_if_consumed_endurance_charge_recently" => [IgniteEffectPrecentIfConsumedEnduranceChargeRecently],
                "invocation_skill_maximum_energy_+%" => [InvocationSkillMaximumEnergyPrecent],
                "invocation_spell_chance_to_cost_half_energy_%" => [InvocationSpellChanceToCostHalfEnergyPrecent],
                "invocation_spell_damage_+%" => [InvocationSpellDamagePrecent],
                "leech_%_is_instant" => [LeechPrecentIsInstant],
                "life_gained_on_attack_hit_if_crit_recently" => [LifeGainedOnAttackHitIfCritRecently],
                "life_gained_on_attack_hit_vs_cursed_enemies" => [LifeGainedOnAttackHitVsCursedEnemies],
                "life_leech_can_overcap_life" => [LifeLeechCanOvercapLife],
                "life_regeneration_per_minute_%_while_frozen" => [LifeRegenerationPerMinutePrecentWhileFrozen],
                "life_regeneration_rate_+%_while_moving" => [LifeRegenerationRatePrecentWhileMoving],
                "life_regeneration_rate_+%_while_using_life_flask" => [LifeRegenerationRatePrecentWhileUsingLifeFlask],
                "lightning_damage_+%_if_lightning_infusion_collected_last_8_seconds" => [LightningDamagePrecentIfLightningInfusionCollectedLast8Seconds],
                "lightning_damage_can_ignite" => [LightningDamageCanIgnite],
                "lightning_damage_taken_goes_to_life_over_4_seconds_%" => [LightningDamageTakenGoesToLifeOver4SecondsPrecent],
                "mana_%_gained_on_block" => [ManaPrecentGainedOnBlock],
                "mana_%_to_gain_as_armour" => [ManaPrecentToGainAsArmour],
                "mana_cost_efficiency_+%_if_dodge_rolled_recently" => [ManaCostEfficiencyPrecentIfDodgeRolledRecently],
                "mana_gained_on_attack_hit_vs_cursed_enemies" => [ManaGainedOnAttackHitVsCursedEnemies],
                "mana_regeneration_rate_+%_while_shocked" => [ManaRegenerationRatePrecentWhileShocked],
                "mana_regeneration_rate_+%_while_stationary" => [ManaRegenerationRatePrecentWhileStationary],
                "mana_reservation_efficiency_-2%_per_1" => [ManaReservationEfficiencyPrecentPer1],
                "mark_skill_gem_level_+" => [MarkSkillGemLevels],
                "marked_enemy_damage_taken_+%" => [MarkedEnemyTakesIncreasedDamage],
                "max_fortification_+1_per_5" => [MaxFortificationPer5],
                "max_puppet_master_stacks_+" => [MaxPuppetMasterStacksPer5],
                "maximum_added_cold_damage_per_frenzy_charge" => [MaximumAddedColdDamagePerFrenzyCharge],
                "maximum_added_fire_damage_if_blocked_recently" => [MaximumAddedFireDamageIfBlockedRecently],
                "maximum_energy_shield_from_body_armour_+%" => [MaximumEnergyShieldFromBodyArmourPrecent],
                "maximum_ward_+%" => [MaximumWardPrecent],
                "melee_attack_skills_additional_totems_allowed" => [MeleeAttackSkillsAdditionalTotemsAllowed],
                "melee_attacks_number_of_additional_projectiles" => [MeleeAttacksNumberOfAdditionalProjectiles],
                "minimum_added_cold_damage_per_frenzy_charge" => [MinimumAddedColdDamagePerFrenzyCharge],
                "minimum_added_fire_damage_if_blocked_recently" => [MinimumAddedFireDamageIfBlockedRecently],
                "minion_area_of_effect_+%" => [MinionAreaOfEffectPrecent],
                "minion_armour_break_physical_damage_%_dealt_as_armour_break" => [MinionArmourBreakPhysicalDamagePrecent],
                "minion_base_damaging_ailment_effect_+%" => [MinionBaseDamagingAilmentEffectPrecent],
                "minion_chance_to_fire_1_additional_projectile_%_with_rollover" => [MinionChanceToFire1AdditionalProjectilePrecent],

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
