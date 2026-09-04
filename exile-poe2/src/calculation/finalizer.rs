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
