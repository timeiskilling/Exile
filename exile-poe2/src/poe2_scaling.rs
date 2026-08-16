//! Auto-generated from mods.json (item domain) — verified per-resource scaling stats.
//! Regenerate with the extraction script if the mod database changes.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Poe2Resource {
    /// `block_chance` (DERIVED — needs Finalizer base+% formula)
    BlockChance,
    /// `increased_item_found_quantity` (flat pool — Accumulated)
    IncreasedItemFoundQuantity,
    /// `overcapped_block_chance` (DERIVED — needs Finalizer base+% formula)
    OvercappedBlockChance,
    /// `armour_break` (DERIVED — needs Finalizer base+% formula)
    ArmourBreak,
    /// `quality` (flat pool — Accumulated)
    Quality,
    /// `max_life_with_non_channelling_skills` (DERIVED — needs Finalizer base+% formula)
    MaxLifeWithNonChannellingSkills,
    /// `max_mana_with_non_channelling_skills` (DERIVED — needs Finalizer base+% formula)
    MaxManaWithNonChannellingSkills,
    /// `devotion` (flat pool — Accumulated)
    Devotion,
    /// `dexterity` (flat pool — Accumulated)
    Dexterity,
    /// `intelligence` (flat pool — Accumulated)
    Intelligence,
    /// `spirit` (flat pool — Accumulated)
    Spirit,
    /// `accuracy` (DERIVED — needs Finalizer base+% formula)
    Accuracy,
    /// `mana_spent_recently` (RUNTIME — not derivable from gear, needs external assumption)
    ManaSpentRecently,
    /// `strength` (flat pool — Accumulated)
    Strength,
    /// `evasion` (DERIVED — needs Finalizer base+% formula)
    Evasion,
    /// `cold_resistance` (DERIVED — needs Finalizer base+% formula)
    ColdResistance,
    /// `ward_cost` (DERIVED — needs Finalizer base+% formula)
    WardCost,
    /// `of_your_lowest_attribute` (DERIVED — needs Finalizer base+% formula)
    OfYourLowestAttribute,
    /// `active_curse_on_self` (RUNTIME — not derivable from gear, needs external assumption)
    ActiveCurseOnSelf,
    /// `active_minion` (RUNTIME — not derivable from gear, needs external assumption)
    ActiveMinion,
    /// `active_undead_minion` (RUNTIME — not derivable from gear, needs external assumption)
    ActiveUndeadMinion,
    /// `broken_face` (RUNTIME — not derivable from gear, needs external assumption)
    BrokenFace,
    /// `cold_resistance_above_75` (DERIVED — needs Finalizer base+% formula)
    ColdResistanceAbove75,
    /// `different_command_skills_used_in_last_15_seconds` (RUNTIME — not derivable from gear, needs external assumption)
    DifferentCommandSkillsUsedInLast15Seconds,
    /// `endurance_charge` (flat pool — Accumulated)
    EnduranceCharge,
    /// `enemy_elemental_ailment` (RUNTIME — not derivable from gear, needs external assumption)
    EnemyElementalAilment,
    /// `equipped_corrupted_item` (flat pool — Accumulated)
    EquippedCorruptedItem,
    /// `equipped_unique` (flat pool — Accumulated)
    EquippedUnique,
    /// `fragile_regrowth` (RUNTIME — not derivable from gear, needs external assumption)
    FragileRegrowth,
    /// `frenzy_charge` (flat pool — Accumulated)
    FrenzyCharge,
    /// `level` (flat pool — Accumulated)
    Level,
    /// `lightning_resistance_above_75` (DERIVED — needs Finalizer base+% formula)
    LightningResistanceAbove75,
    /// `pierced_enemy` (RUNTIME — not derivable from gear, needs external assumption)
    PiercedEnemy,
    /// `poison_up_to_75` (RUNTIME — not derivable from gear, needs external assumption)
    PoisonUpTo75,
    /// `power_charge` (flat pool — Accumulated)
    PowerCharge,
    /// `rage` (RUNTIME — not derivable from gear, needs external assumption)
    Rage,
    /// `spell_crit_dealt_recently` (RUNTIME — not derivable from gear, needs external assumption)
    SpellCritDealtRecently,
    /// `stackable_unique_jewel` (flat pool — Accumulated)
    StackableUniqueJewel,
    /// `target_power` (RUNTIME — not derivable from gear, needs external assumption)
    TargetPower,
    /// `white_socket_on_item` (flat pool — Accumulated)
    WhiteSocketOnItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceSource {
    /// Flat pool: safe to read straight out of the accumulator's global_stats
    /// once Base-phase effects (or an item-introspection pass, for things like
    /// EquippedCorruptedItem) have been applied — no % formula involved.
    Accumulated,
    /// Only has a final value after the Finalizer computes base + increased% for
    /// that stat. Reading it mid-apply from a raw accumulator sum is wrong.
    Derived,
    /// Combat-log / instantaneous runtime state (recent crits, active minions,
    /// pierced enemies this hit...). Not computable from gear at all — route
    /// through the same external assumption/config mechanism you already use
    /// for EffectConditionEvaluator's Context, not through the accumulator.
    RuntimeAssumption,
}

impl Poe2Resource {
    pub const fn source(self) -> ResourceSource {
        match self {
            Poe2Resource::BlockChance => ResourceSource::Derived,
            Poe2Resource::IncreasedItemFoundQuantity => ResourceSource::Accumulated,
            Poe2Resource::OvercappedBlockChance => ResourceSource::Derived,
            Poe2Resource::ArmourBreak => ResourceSource::Derived,
            Poe2Resource::Quality => ResourceSource::Accumulated,
            Poe2Resource::MaxLifeWithNonChannellingSkills => ResourceSource::Derived,
            Poe2Resource::MaxManaWithNonChannellingSkills => ResourceSource::Derived,
            Poe2Resource::Devotion => ResourceSource::Accumulated,
            Poe2Resource::Dexterity => ResourceSource::Accumulated,
            Poe2Resource::Intelligence => ResourceSource::Accumulated,
            Poe2Resource::Spirit => ResourceSource::Accumulated,
            Poe2Resource::Accuracy => ResourceSource::Derived,
            Poe2Resource::ManaSpentRecently => ResourceSource::RuntimeAssumption,
            Poe2Resource::Strength => ResourceSource::Accumulated,
            Poe2Resource::Evasion => ResourceSource::Derived,
            Poe2Resource::ColdResistance => ResourceSource::Derived,
            Poe2Resource::WardCost => ResourceSource::Derived,
            Poe2Resource::OfYourLowestAttribute => ResourceSource::Derived,
            Poe2Resource::ActiveCurseOnSelf => ResourceSource::RuntimeAssumption,
            Poe2Resource::ActiveMinion => ResourceSource::RuntimeAssumption,
            Poe2Resource::ActiveUndeadMinion => ResourceSource::RuntimeAssumption,
            Poe2Resource::BrokenFace => ResourceSource::RuntimeAssumption,
            Poe2Resource::ColdResistanceAbove75 => ResourceSource::Derived,
            Poe2Resource::DifferentCommandSkillsUsedInLast15Seconds => ResourceSource::RuntimeAssumption,
            Poe2Resource::EnduranceCharge => ResourceSource::Accumulated,
            Poe2Resource::EnemyElementalAilment => ResourceSource::RuntimeAssumption,
            Poe2Resource::EquippedCorruptedItem => ResourceSource::Accumulated,
            Poe2Resource::EquippedUnique => ResourceSource::Accumulated,
            Poe2Resource::FragileRegrowth => ResourceSource::RuntimeAssumption,
            Poe2Resource::FrenzyCharge => ResourceSource::Accumulated,
            Poe2Resource::Level => ResourceSource::Accumulated,
            Poe2Resource::LightningResistanceAbove75 => ResourceSource::Derived,
            Poe2Resource::PiercedEnemy => ResourceSource::RuntimeAssumption,
            Poe2Resource::PoisonUpTo75 => ResourceSource::RuntimeAssumption,
            Poe2Resource::PowerCharge => ResourceSource::Accumulated,
            Poe2Resource::Rage => ResourceSource::RuntimeAssumption,
            Poe2Resource::SpellCritDealtRecently => ResourceSource::RuntimeAssumption,
            Poe2Resource::StackableUniqueJewel => ResourceSource::Accumulated,
            Poe2Resource::TargetPower => ResourceSource::RuntimeAssumption,
            Poe2Resource::WhiteSocketOnItem => ResourceSource::Accumulated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Poe2Scaling {
    pub resource: Poe2Resource,
    pub per: u32,
}

/// Maps a verified scaling suffix (including its leading `_per_`, e.g. `_per_frenzy_charge`)
/// to its `Poe2Scaling` descriptor.
pub fn parse_scaling(suffix: &str) -> Poe2Scaling {
    match suffix {
        "_per_1%_block_chance" => Poe2Scaling { resource: Poe2Resource::BlockChance, per: 1 },
        "_per_1%_increased_item_found_quantity" => Poe2Scaling { resource: Poe2Resource::IncreasedItemFoundQuantity, per: 1 },
        "_per_1%_overcapped_block_chance" => Poe2Scaling { resource: Poe2Resource::OvercappedBlockChance, per: 1 },
        "_per_10%_armour_break" => Poe2Scaling { resource: Poe2Resource::ArmourBreak, per: 10 },
        "_per_10%_quality" => Poe2Scaling { resource: Poe2Resource::Quality, per: 10 },
        "_per_100_max_life_with_non_channelling_skills" => Poe2Scaling { resource: Poe2Resource::MaxLifeWithNonChannellingSkills, per: 100 },
        "_per_100_max_mana_with_non_channelling_skills" => Poe2Scaling { resource: Poe2Resource::MaxManaWithNonChannellingSkills, per: 100 },
        "_per_10_devotion" => Poe2Scaling { resource: Poe2Resource::Devotion, per: 10 },
        "_per_10_dex" => Poe2Scaling { resource: Poe2Resource::Dexterity, per: 10 },
        "_per_10_int" => Poe2Scaling { resource: Poe2Resource::Intelligence, per: 10 },
        "_per_10_intelligence" => Poe2Scaling { resource: Poe2Resource::Intelligence, per: 10 },
        "_per_10_spirit" => Poe2Scaling { resource: Poe2Resource::Spirit, per: 10 },
        "_per_15_dex" => Poe2Scaling { resource: Poe2Resource::Dexterity, per: 15 },
        "_per_200_accuracy" => Poe2Scaling { resource: Poe2Resource::Accuracy, per: 200 },
        "_per_200_mana_spent_recently" => Poe2Scaling { resource: Poe2Resource::ManaSpentRecently, per: 200 },
        "_per_20_dex" => Poe2Scaling { resource: Poe2Resource::Dexterity, per: 20 },
        "_per_20_spirit" => Poe2Scaling { resource: Poe2Resource::Spirit, per: 20 },
        "_per_20_strength" => Poe2Scaling { resource: Poe2Resource::Strength, per: 20 },
        "_per_25_dexterity" => Poe2Scaling { resource: Poe2Resource::Dexterity, per: 25 },
        "_per_25_strength" => Poe2Scaling { resource: Poe2Resource::Strength, per: 25 },
        "_per_4%_quality" => Poe2Scaling { resource: Poe2Resource::Quality, per: 4 },
        "_per_450_evasion" => Poe2Scaling { resource: Poe2Resource::Evasion, per: 450 },
        "_per_4_strength" => Poe2Scaling { resource: Poe2Resource::Strength, per: 4 },
        "_per_5%_block_chance" => Poe2Scaling { resource: Poe2Resource::BlockChance, per: 5 },
        "_per_5%_cold_resistance" => Poe2Scaling { resource: Poe2Resource::ColdResistance, per: 5 },
        "_per_50_dex" => Poe2Scaling { resource: Poe2Resource::Dexterity, per: 50 },
        "_per_50_ward_cost" => Poe2Scaling { resource: Poe2Resource::WardCost, per: 50 },
        "_per_5_dex" => Poe2Scaling { resource: Poe2Resource::Dexterity, per: 5 },
        "_per_5_of_your_lowest_attribute" => Poe2Scaling { resource: Poe2Resource::OfYourLowestAttribute, per: 5 },
        "_per_8%_quality" => Poe2Scaling { resource: Poe2Resource::Quality, per: 8 },
        "_per_8_strength" => Poe2Scaling { resource: Poe2Resource::Strength, per: 8 },
        "_per_active_curse_on_self" => Poe2Scaling { resource: Poe2Resource::ActiveCurseOnSelf, per: 1 },
        "_per_active_minion" => Poe2Scaling { resource: Poe2Resource::ActiveMinion, per: 1 },
        "_per_active_undead_minion" => Poe2Scaling { resource: Poe2Resource::ActiveUndeadMinion, per: 1 },
        "_per_broken_face" => Poe2Scaling { resource: Poe2Resource::BrokenFace, per: 1 },
        "_per_cold_resistance_above_75" => Poe2Scaling { resource: Poe2Resource::ColdResistanceAbove75, per: 1 },
        "_per_different_command_skills_used_in_last_15_seconds" => Poe2Scaling { resource: Poe2Resource::DifferentCommandSkillsUsedInLast15Seconds, per: 1 },
        "_per_endurance_charge" => Poe2Scaling { resource: Poe2Resource::EnduranceCharge, per: 1 },
        "_per_enemy_elemental_ailment" => Poe2Scaling { resource: Poe2Resource::EnemyElementalAilment, per: 1 },
        "_per_equipped_corrupted_item" => Poe2Scaling { resource: Poe2Resource::EquippedCorruptedItem, per: 1 },
        "_per_equipped_unique" => Poe2Scaling { resource: Poe2Resource::EquippedUnique, per: 1 },
        "_per_fragile_regrowth" => Poe2Scaling { resource: Poe2Resource::FragileRegrowth, per: 1 },
        "_per_frenzy_charge" => Poe2Scaling { resource: Poe2Resource::FrenzyCharge, per: 1 },
        "_per_level" => Poe2Scaling { resource: Poe2Resource::Level, per: 1 },
        "_per_lightning_resistance_above_75" => Poe2Scaling { resource: Poe2Resource::LightningResistanceAbove75, per: 1 },
        "_per_pierced_enemy" => Poe2Scaling { resource: Poe2Resource::PiercedEnemy, per: 1 },
        "_per_poison_up_to_75%" => Poe2Scaling { resource: Poe2Resource::PoisonUpTo75, per: 1 },
        "_per_power_charge" => Poe2Scaling { resource: Poe2Resource::PowerCharge, per: 1 },
        "_per_rage" => Poe2Scaling { resource: Poe2Resource::Rage, per: 1 },
        "_per_spell_crit_dealt_recently" => Poe2Scaling { resource: Poe2Resource::SpellCritDealtRecently, per: 1 },
        "_per_stackable_unique_jewel" => Poe2Scaling { resource: Poe2Resource::StackableUniqueJewel, per: 1 },
        "_per_target_power" => Poe2Scaling { resource: Poe2Resource::TargetPower, per: 1 },
        "_per_white_socket_on_item" => Poe2Scaling { resource: Poe2Resource::WhiteSocketOnItem, per: 1 },
        other => panic!("unrecognized verified scaling suffix: {other} — regenerate poe2_scaling.rs"),
    }
}
