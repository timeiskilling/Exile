//! Auto-generated from mods.json (item domain) — verified boolean-gate conditions.
//! Regenerate with the extraction script if the mod database changes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Poe2Condition {
    /// `_during_flask_effect`
    FlaskEffect,
    /// `_during_mana_flask_effect`
    ManaFlaskEffect,
    /// `_if_blocked_recently`
    BlockedRecently,
    /// `_if_consumed_endurance_charge_recently`
    ConsumedEnduranceChargeRecently,
    /// `_if_crit_recently`
    CritRecently,
    /// `_if_culled_recently`
    CulledRecently,
    /// `_if_cursed_enemy_killed_recently`
    CursedEnemyKilledRecently,
    /// `_if_enemy_hit_recently`
    EnemyHitRecently,
    /// `_if_fire_infusion_collected_last_8_seconds`
    FireInfusionCollectedLast8Seconds,
    /// `_if_have_been_hit_recently`
    HaveBeenHitRecently,
    /// `_if_have_crit_in_past_8_seconds`
    HaveCritInPast8Seconds,
    /// `_if_have_crit_recently`
    HaveCritRecently,
    /// `_if_have_not_crit_recently`
    HaveNotCritRecently,
    /// `_if_lightning_infusion_collected_last_8_seconds`
    LightningInfusionCollectedLast8Seconds,
    /// `_if_lost_archon_in_past_6_seconds`
    LostArchonInPast6Seconds,
    /// `_if_not_been_hit_recently`
    NotBeenHitRecently,
    /// `_if_not_crit_recently`
    NotCritRecently,
    /// `_if_other_ring_is_elder_item`
    OtherRingIsElderItem,
    /// `_if_other_ring_is_shaper_item`
    OtherRingIsShaperItem,
    /// `_if_reloaded_recently`
    ReloadedRecently,
    /// `_if_sprinting`
    Sprinting,
    /// `_if_you_dodge_rolled_recently`
    YouDodgeRolledRecently,
    /// `_if_you_have_beast_minion`
    YouHaveBeastMinion,
    /// `_if_you_have_frozen_enemy_recently`
    YouHaveFrozenEnemyRecently,
    /// `_if_you_have_shocked_recently`
    YouHaveShockedRecently,
    /// `_if_you_have_used_a_cold_skill_recently`
    YouHaveUsedAColdSkillRecently,
    /// `_if_you_have_used_a_fire_skill_recently`
    YouHaveUsedAFireSkillRecently,
    /// `_if_youve_dealt_melee_hit_recently`
    YouveDealtMeleeHitRecently,
    /// `_if_youve_dealt_projectile_attack_hit_recently`
    YouveDealtProjectileAttackHitRecently,
    /// `_when_in_main_hand`
    InMainHand,
    /// `_when_in_off_hand`
    InOffHand,
    /// `_when_not_on_low_mana`
    NotOnLowMana,
    /// `_when_on_full_life`
    OnFullLife,
    /// `_when_on_low_life`
    OnLowLife,
    /// `_while_affected_by_herald_of_agony`
    AffectedByHeraldOfAgony,
    /// `_while_affected_by_herald_of_ash`
    AffectedByHeraldOfAsh,
    /// `_while_affected_by_herald_of_ice`
    AffectedByHeraldOfIce,
    /// `_while_affected_by_herald_of_purity`
    AffectedByHeraldOfPurity,
    /// `_while_affected_by_herald_of_thunder`
    AffectedByHeraldOfThunder,
    /// `_while_affected_by_malevolence`
    AffectedByMalevolence,
    /// `_while_at_maximum_frenzy_charges`
    AtMaximumFrenzyCharges,
    /// `_while_at_maximum_power_charges`
    AtMaximumPowerCharges,
    /// `_while_chilled_or_frozen`
    ChilledOrFrozen,
    /// `_while_frozen`
    Frozen,
    /// `_while_have_onslaught`
    HaveOnslaught,
    /// `_while_ignited`
    Ignited,
    /// `_while_leeching`
    Leeching,
    /// `_while_missing_ward`
    MissingWard,
    /// `_while_moving`
    Moving,
    /// `_while_not_cursed`
    NotCursed,
    /// `_while_off_hand_is_empty`
    OffHandIsEmpty,
    /// `_while_on_full_mana`
    OnFullMana,
    /// `_while_onslaught_is_active`
    OnslaughtIsActive,
    /// `_while_phasing`
    Phasing,
    /// `_while_poisoned`
    Poisoned,
    /// `_while_shocked`
    Shocked,
    /// `_while_spider`
    Spider,
    /// `_while_stationary`
    Stationary,
    /// `_while_surrounded`
    Surrounded,
    /// `_while_unarmed`
    Unarmed,
    /// `_while_using_flask`
    UsingFlask,
}

/// Maps a verified condition suffix (including its leading marker, e.g. `_when_on_full_life`)
/// to its `Poe2Condition` variant. Called only after `find_valid_split` has confirmed
/// the base id exists independently in the database.
pub fn parse_condition(suffix: &str) -> Poe2Condition {
    match suffix {
        "_during_flask_effect" => Poe2Condition::FlaskEffect,
        "_during_mana_flask_effect" => Poe2Condition::ManaFlaskEffect,
        "_if_blocked_recently" => Poe2Condition::BlockedRecently,
        "_if_consumed_endurance_charge_recently" => Poe2Condition::ConsumedEnduranceChargeRecently,
        "_if_crit_recently" => Poe2Condition::CritRecently,
        "_if_culled_recently" => Poe2Condition::CulledRecently,
        "_if_cursed_enemy_killed_recently" => Poe2Condition::CursedEnemyKilledRecently,
        "_if_enemy_hit_recently" => Poe2Condition::EnemyHitRecently,
        "_if_fire_infusion_collected_last_8_seconds" => Poe2Condition::FireInfusionCollectedLast8Seconds,
        "_if_have_been_hit_recently" => Poe2Condition::HaveBeenHitRecently,
        "_if_have_crit_in_past_8_seconds" => Poe2Condition::HaveCritInPast8Seconds,
        "_if_have_crit_recently" => Poe2Condition::HaveCritRecently,
        "_if_have_not_crit_recently" => Poe2Condition::HaveNotCritRecently,
        "_if_lightning_infusion_collected_last_8_seconds" => Poe2Condition::LightningInfusionCollectedLast8Seconds,
        "_if_lost_archon_in_past_6_seconds" => Poe2Condition::LostArchonInPast6Seconds,
        "_if_not_been_hit_recently" => Poe2Condition::NotBeenHitRecently,
        "_if_not_crit_recently" => Poe2Condition::NotCritRecently,
        "_if_other_ring_is_elder_item" => Poe2Condition::OtherRingIsElderItem,
        "_if_other_ring_is_shaper_item" => Poe2Condition::OtherRingIsShaperItem,
        "_if_reloaded_recently" => Poe2Condition::ReloadedRecently,
        "_if_sprinting" => Poe2Condition::Sprinting,
        "_if_you_dodge_rolled_recently" => Poe2Condition::YouDodgeRolledRecently,
        "_if_you_have_beast_minion" => Poe2Condition::YouHaveBeastMinion,
        "_if_you_have_frozen_enemy_recently" => Poe2Condition::YouHaveFrozenEnemyRecently,
        "_if_you_have_shocked_recently" => Poe2Condition::YouHaveShockedRecently,
        "_if_you_have_used_a_cold_skill_recently" => Poe2Condition::YouHaveUsedAColdSkillRecently,
        "_if_you_have_used_a_fire_skill_recently" => Poe2Condition::YouHaveUsedAFireSkillRecently,
        "_if_youve_dealt_melee_hit_recently" => Poe2Condition::YouveDealtMeleeHitRecently,
        "_if_youve_dealt_projectile_attack_hit_recently" => Poe2Condition::YouveDealtProjectileAttackHitRecently,
        "_when_in_main_hand" => Poe2Condition::InMainHand,
        "_when_in_off_hand" => Poe2Condition::InOffHand,
        "_when_not_on_low_mana" => Poe2Condition::NotOnLowMana,
        "_when_on_full_life" => Poe2Condition::OnFullLife,
        "_when_on_low_life" => Poe2Condition::OnLowLife,
        "_while_affected_by_herald_of_agony" => Poe2Condition::AffectedByHeraldOfAgony,
        "_while_affected_by_herald_of_ash" => Poe2Condition::AffectedByHeraldOfAsh,
        "_while_affected_by_herald_of_ice" => Poe2Condition::AffectedByHeraldOfIce,
        "_while_affected_by_herald_of_purity" => Poe2Condition::AffectedByHeraldOfPurity,
        "_while_affected_by_herald_of_thunder" => Poe2Condition::AffectedByHeraldOfThunder,
        "_while_affected_by_malevolence" => Poe2Condition::AffectedByMalevolence,
        "_while_at_maximum_frenzy_charges" => Poe2Condition::AtMaximumFrenzyCharges,
        "_while_at_maximum_power_charges" => Poe2Condition::AtMaximumPowerCharges,
        "_while_chilled_or_frozen" => Poe2Condition::ChilledOrFrozen,
        "_while_frozen" => Poe2Condition::Frozen,
        "_while_have_onslaught" => Poe2Condition::HaveOnslaught,
        "_while_ignited" => Poe2Condition::Ignited,
        "_while_leeching" => Poe2Condition::Leeching,
        "_while_missing_ward" => Poe2Condition::MissingWard,
        "_while_moving" => Poe2Condition::Moving,
        "_while_not_cursed" => Poe2Condition::NotCursed,
        "_while_not_on_low_mana" => Poe2Condition::NotOnLowMana,
        "_while_off_hand_is_empty" => Poe2Condition::OffHandIsEmpty,
        "_while_on_full_mana" => Poe2Condition::OnFullMana,
        "_while_on_low_life" => Poe2Condition::OnLowLife,
        "_while_onslaught_is_active" => Poe2Condition::OnslaughtIsActive,
        "_while_phasing" => Poe2Condition::Phasing,
        "_while_poisoned" => Poe2Condition::Poisoned,
        "_while_shocked" => Poe2Condition::Shocked,
        "_while_spider" => Poe2Condition::Spider,
        "_while_sprinting" => Poe2Condition::Sprinting,
        "_while_stationary" => Poe2Condition::Stationary,
        "_while_surrounded" => Poe2Condition::Surrounded,
        "_while_unarmed" => Poe2Condition::Unarmed,
        "_while_using_flask" => Poe2Condition::UsingFlask,
        other => panic!("unrecognized verified condition suffix: {other} — regenerate poe2_condition.rs"),
    }
}
