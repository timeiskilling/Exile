use crate::ModType;
use crate::effect::planning::{Poe2ConflictKey, Poe2EffectPhase, Poe2SelectionKey};
use crate::poe2_condition::Poe2Condition;
use crate::poe2_scaling::Poe2Scaling;
use crate::repoe_parse::{GenerationType, HashedTagWeight, ItemClass, Properties, Requirements};
use ahash::AHasher;
use ahash::HashSet;
use exile_core::game::{Game, ModifierDefinitionIdentity};
use std::hash::{Hash, Hasher};

pub fn hash_string(s: &str) -> u64 {
    let mut hasher = AHasher::default();
    s.hash(&mut hasher);
    hasher.finish()
}

pub struct Poe2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemRarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    MainHand,
    OffHand,
    BodyArmour,
    Helmet,
    Gloves,
    Boots,
    Amulet,
    Ring1,
    Ring2,
    Belt,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Poe2StatModifierKind {
    Plain,
    Conditional(Poe2Condition),
    Scaled(Poe2Scaling),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatBucket {
    Life,
    LifePercent,
    Mana,
    ManaPercent,
    EnergyShield,
    MaximumEnergyShieldPercent,
    Spirit,
    Ward,
    Armour,
    ArmourPercent,
    Evasion,
    EvasionPercent,
    Block,
    ThornsPhysicalMin,
    ThornsPhysicalMax,
    FireResistance,
    ColdResistance,
    LightningResistance,
    ChaosResistance,
    Strength,
    Dexterity,
    Intelligence,
    ChaosDamagePercent,
    StunThreshold,
    ReducedAttributeRequirementsPercent,
    MovementSpeedPercent,
    GlobalAttackPhysicalMin,
    GlobalAttackPhysicalMax,
    GlobalAttackFireMin,
    GlobalAttackFireMax,
    GlobalAttackColdMin,
    GlobalAttackColdMax,
    GlobalAttackLightningMin,
    GlobalAttackLightningMax,
    AttackDamageIncreasePrecent,
    AlliesInPresenceAttackPhysicalMin,
    AlliesInPresenceAttackPhysicalMax,
    AlliesInPresenceAttackFireMin,
    AlliesInPresenceAttackFireMax,
    AlliesInPresenceAttackColdMin,
    AlliesInPresenceAttackColdMax,
    AlliesInPresenceAttackLightningMin,
    AlliesInPresenceAttackLightningMax,
    AlliesInPresenceAttackChaosMin,
    AlliesInPresenceAttackChaosMax,
    AlliesInPresenceDamageIncreasePrecent,
    AlliesInPresenceAttackSpeedIncreasePrecent,
    AlliesInPresenceAccuracyRating,
    AccuracyRating,
    AccuracyRatingPercent,
    ChanceToNotConsumeExertedAttack,
    SpellDamageIncreasePrecent,
    SpellPhysicalDamageIncreasePrecent,
    FireDamageIncreasePrecent,
    ColdDamageIncreasePrecent,
    LightningDamageIncreasePrecent,
    ChaosDamageIncreasePrecent,
    PhysicalDamageIncreasePrecent,
    TrapDamageIncreasePrecent,
    SpellSkillGemLevelIncrease,
    FireSpellSkillGemLevelIncrease,
    ColdSpellSkillGemLevelIncrease,
    LightningSpellSkillGemLevelIncrease,
    ChaosSpellSkillGemLevelIncrease,
    PhysicalSpellSkillGemLevelIncrease,
    MinionSkillGemLevelIncrease,
    TrapSkillGemLevelIncrease,
    MeleeSkillGemLevelIncrease,
    ProjectileSkillGemLevelIncrease,
    LiferRegenPerMinute,
    LifeRegenPerMinutePercent,
    AlliesInPresenceLifeRegenPerMinute,
    ManaRegenerationRatePercent,
    BaseLifeLeechFromPhysicalAttackDamage,
    BaseManaLeechFromPhysicalAttackDamage,
    BaseLifeGainedOnEnemyDeath,
    BaseManaGainedOnEnemyDeath,
    BaseLifeGainedOnEnemyHit,
    BaseManaGainedOnEnemyHit,
    AttackSpeedPercent,
    BaseCastSpeedIncreasePrecent,
    AlliesInPresenceCastSpeedIncreasePrecent,
    CriticalStrikeChanceIncreasePrecent,
    TrapThrowingSpeedIncreasePrecent,
    SpellCriticalStrikeChanceIncreasePrecent,
    AttackCriticalStrikeChanceIncreasePrecent,
    TrapCriticalStrikeChanceIncreasePrecent,
    AlliesInPresenceCriticalStrikeChanceIncreasePrecent,
    BaseCriticalStrikeMultiplierIncreasePrecent,
    BaseSpellCriticalStrikeMultiplierIncreasePrecent,
    AttackCriticalStrikeMultiplierIncreasePrecent,
    AttackCriticalStrikeMultiplierIncrease,
    TrapCriticalStrikeMultiplierIncrease,
    AlliesInPresenceCriticalStrikeMultiplierIncrease,
    BaseItemFoundRarityIncreasePrecent,
    LightRadiusIncreasePrecent,
    SelfBleedDurationDecreasePrecent,
    SelfPoisonDurationDecreasePrecent,
    BaseIgniteDurationDecreasePrecent,
    BaseSelfShockDurationDecreasePrecent,
    BaseSelfChillDurationDecreasePrecent,
    BaseSelfFreezeDurationDecreasePrecent,
    ReduceCriticalStrikeMultiplierToSelf,
    PhysicalDamageReductionPrecent,
    FireResistanceMax,
    ColdResistanceMax,
    LightningResistanceMax,
    ChaosResistanceMax,
    AllElementalResistanceMax,
    EnergyShieldRechargeRateIncreasePrecent,
    EnergyShieldDelayDecreasePrecent,
    ArmourAppliesToElementalDamage,
    ArmourPercentAppliesToChaosDamage,
    EvasionAppliesToDeflection,
    BaseDeflectionRatingPercentOfArmour,
    DeflectDamageTaken,
    PercentEvasionRatingAsExtraArmour,
    BaseChanceToPierce,
    BaseNumberOfCrossbowBolts,
    FlaskLifeRecoveryRateIncreasePrecent,
    LifeFlaskChargesGainedIncreasePrecent,
    FlaskManaRecoveryRateIncreasePrecent,
    ManaFlaskChargesGainedIncreasePrecent,
    FlaskRecoveryAmountPercentToRecoverInstantly,
    GenerateXChargesForAnyFlaskPerMinute,
    MaximumManaGainedOnKillPercent,
    BaseManaLeechAmountIncreasePrecent,
    MovementSpeedPenaltyReductionPercent,
    BaseDamageRemovedFromManaBeforeLifePercent,
    SelfStatusAilmentDurationReductionPercent,
    CorruptedSkillGemLevelIncrease,
    WardRegenerationRateIncreasePrecent,
    BaseIgniteIncereasedMagnitudePrecent,
    BaseDamageTakenIncreasePrecent,
    IncreaseEvasionPercentArmourPercentEnergyShieldPercentFromShield,
    StrikesDealMeleeSplash,
    WarCryEmpowersNextXMeleeAttacks,
    GainXRageOnMeleeHit,
    GainXRageWhenGetByEnemyHit,
    ShockEffectIncreasePrecent,
    MaximumRage,
    MinionAccuracyRatingIncreasePrecent,
    MinionSkillAreaOfEffectIncreasePrecent,
    MinionChaosResistanceIncreasePrecent,
    MinionCriticalStrikeChanceIncreasePrecent,
    MinionCriticalStrikeMultiplierIncreasePrecent,
    MinionAttackAndCastSpeedIncreasePrecent,
    MeleeDamageIncreasePrecent,
    MarkEffectIncreasePrecent,
    MarkSkillDurationIncreasePrecent,
    MarkUseSpeedIncreasePrecent,
    SpellsCostLifeInsteadOfMana,
    ProjectileAttackRangePrecent,
    ChanceToFire1AdditionalProjectileWithRolloverWithBowAttacks,
    ProjectileSpeedIncreasePrecentForCrossbowSkills,
    GrenadeSkillNumberOfAdditionalProjectiles,
    AdditionalBallistaTotemsAllowed,
    GrenadeSkillCooldownCount,
    GrenadeSkillCooldownSpeedIncreasePrecent,
    PlacingTrapsCooldownRecoveryForThrowingIncreasePrecent,
    AilmentChanceIncreasePrecent,
    AilmentEffectIncreasePrecent,
    BaseSkillAreaOfEffectIncreasePrecent,
    ArmourBreakAmountIncreasePrecent,
    AuraEffectIncreasePrecent,
    AxeDamageIncreasePrecent,
    AxeAttackSpeedIncreasePrecent,
    BaseChanceToInflictBleeding,
    BaseBleedDurationIncreasePrecent,
    BlindEffectIncreasePrecent,
    ChanceToBlindOnHitWithAttacks,
    DamageToRareAndUniqueEnemiesIncreasePrecent,
    ProjectileChanceToChain1ExtraTimeFromTerrainIncreasePrecent,
    ColdResistancePenetration,
    BaseCooldownSpeedIncreasePrecent,
    DamageIfYouHaveConsumedACorpseRecentlyIncreasePrecent,
    CriticalHitDamagingAilmentEffectIncreasePrecent,
    CrossbowDamageIncreasePrecent,
    ReloadSpeedIncreasePrecent,
    CrossbowAttackSpeedIncreasePrecent,
    CurseAreaOfEffectIncreasePrecent,
    CurseDelayIncreasePrecent,
    BaseCurseDurationIncreasePrecent,
    CurseEffectIncreasePrecent,
    DaggerCriticalStrikeChanceIncreasePrecent,
    DaggerDamageIncreasePrecent,
    DaggerAttackSpeedIncreasePrecent,
    DamageAgainstEnemiesWithFullyBrokenArmourIncreasePrecent,
    DamagingAilmentDurationIncreasePrecent,
    BaseChanceToDazeIncreasePrecent,
    DebuffTimePassedIncreasePrecent,
    IgniteShockChillDurationIncreasePrecent,
    ElementalDamageIncreasePrecent,
    EmpoweredAttackDamageIncreasePrecent,
    EnergyGeneratedIncreasePrecent,
    DamagingAilmentsDealDamageIncreasePrecent,
    FireResistancePenetration,
    FlailCriticalStrikeChanceIncreasePrecent,
    FlailDamageIncreasePrecent,
    EnergyShieldFromFocusIncreasePrecent,
    ChanceToForkExtraProjectileIncreasePrecent,
    FreezeThresholdIncreasePrecent,
    DamageWithHeraldSkillsIncreasePrecent,
    SkillEffectDurationIncreasePrecent,
    KnockbackDistanceIncreasePrecent,
    BaseSkillCostLifeInsteadOfManaIncreasePrecent,
    BaseLifeLeechAmountIncreasePrecent,
    RecoverMaximumLifeOnKillIncreasePrecent,
    DamageTakenGoesToLifeOver4SecondsIncreasePrecent,
    LifeRegenerationRateIncreasePrecent,
    LightningResistancePenetration,
    MaceDamageIncreasePrecent,
    MaceHitDamageStunMultiplierIncreasePrecent,
    AccuracyRatingIncreaseWithBowPrecent,
    BowDamageIncreasePrecent,
    BowAttackSpeedIncreasePrecent,
    ChillDurationIncreasePrecent,
    BlockChanceIncreasePrecent,
    ArmourBreakDurationIncreasePrecent,
    MinionDamageIncreasePrecent,
    AdditionalBlockIncreasePrecent,
    FasterBleedDamage,
    AilmentThresholdPercent,
    StunThresholdPercent,
    CharmDurationIncreasePrecent,
    FlaskChargesGainedIncreasePrecent,
    FlaskDurationIncreasePrecent,
    FlaskChargesReducedUsedIncreasePrecent,
    CharmChargesGainedIncreasePrecent,
    CharmChargesReducedUsedIncreasePrecent,
    IgniteChanceIncreasePrecent,
    DamageWhileUsingCharmIncreasePrecent,
    HitDamageFreezeMultiplierIncreasePrecent,
    ShockChanceIncreasePrecent,
    ShockDurationIncreasePrecent,
    BaseProjectileSpeedIncreasePrecent,
    DamageTakenGoesToLifeOver4Seconds,
    DamageTakenGoesToMana,
    ElementalDamageWithAttackSkillsIncreasePrecent,
    NonSkillBaseAllDamageToGainAsFire,
    NonSkillBaseAllDamageToGainAsCold,
    NonSkillBaseAllDamageToGainAsLightning,
    NonSkillBaseAllDamageToGainAsChaos,
    DamageWithBowSkillsIncreasePrecent,
    PresenceAreaIncreasePrecent,
    MinionMaximumLifeIncreasePrecent,
    MinionFireResistance,
    MinionColdResistance,
    MinionLightningResistance,
    MinionChaosResistance,
    MinionResummonSpeedIncreasePrecent,
    MinionAdditionalPhysicalDamageReductionIncreasePrecent,
    TrapTriggerRadiusIncreasePrecent,
    CharmRecoverXLifeWhenUsed,
    CharmRecoverXManaWhenUsed,
    CharmGainXGuardForDuration,
    HitDamageStunMultiplierIncreasePrecent,
    ChanceToPoisonOnHitWithAttacks,
    ChanceBleedOnHitWithAttacks,
    BaseArrowSpeedIncreasePrecent,
    BaseSlowPotencyPercent,
    OfferingDurationIncreasePrecent,
    OfferingLifeIncreasePrecent,
    HitDamagePinMultiplierIncreasePrecent,
    BaseChanceToPoisonOnHitIncreasePrecent,
    BasePoisonEffectIncreasePrecent,
    BasePoisonDurationIncreasePrecent,
    ProjectileDamageIncreasePrecent,
    QuarterstaffDamageIncreasePrecent,
    QuarterstaffHitDamageFreezeMultiplierIncreasePrecent,
    QuarterstaffAttackSpeedIncreasePrecent,
    QuiverModEffectIncreasePrecent,
    SpearAttackSpeedIncreasePrecent,
    SpearCriticalStrikeMultiplierIncreasePrecent,
    SpearDamageIncreasePrecent,
    StunThresholdFromMaximumEnergyShieldPercent,
    AilmentThresholdFromMaximumEnergyShieldPercent,
    StunThresholdWhenNotStunnedRecentlyPercent,
    SwordDamageIncreasePrecent,
    BaseBleedingEffectPercent,
    SwordAttackSpeedIncreasePrecent,
    ThornsDamageIncreasePrecent,
    TotemDamageIncreasePrecent,
    TotemLifeIncreasePrecent,
    SummonTotemCastSpeedIncreasePrecent,
}

pub fn stat_id_to_bucket(stat_id: &str) -> Option<StatBucket> {
    match stat_id {
        "chaos_damage_+%" => Some(StatBucket::ChaosDamagePercent),
        "additional_strength" => Some(StatBucket::Strength),

        _ => None,
    }
}

pub fn classify_bucket(tags: &[String]) -> Option<StatBucket> {
    const RULES: &[(&str, StatBucket)] = &[
        ("fire_resistance", StatBucket::FireResistance),
        ("cold_resistance", StatBucket::ColdResistance),
        ("lightning_resistance", StatBucket::LightningResistance),
        ("chaos_resistance", StatBucket::ChaosResistance),
        ("energy_shield", StatBucket::EnergyShield),
        ("life", StatBucket::Life),
        ("mana", StatBucket::Mana),
        ("armour", StatBucket::Armour),
        ("evasion", StatBucket::Evasion),
        ("block", StatBucket::Block),
    ];
    RULES
        .iter()
        .find(|(tag, _)| tags.iter().any(|t| t == tag))
        .map(|(_, b)| *b)
}

#[derive(Debug, Clone)]
pub struct Poe2ItemState {
    pub item_level: u16,
    pub quality: u16,
    pub rarity: ItemRarity,
    pub is_corrupted: bool,

    pub base_name: String,
    pub drop_level: u16,
    pub properties: Properties,
    pub requirements: Option<Requirements>,
    pub tags: HashSet<u64>,
    pub equip_slot: EquipSlot,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Poe2ModifierId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModOrigin {
    Crafted,
    Fractured,
    Rune,
    Dropped,
    Corrupted,
}

#[derive(Debug, Clone)]
pub struct Poe2ModifierStat {
    pub id_hash: u64,
    pub min: i64,
    pub max: i64,
    pub is_local: bool,
    pub kind: Poe2StatModifierKind,
    pub phase: Poe2EffectPhase,
    pub buckets: Vec<StatBucket>,
    pub conflict_key: Option<Poe2ConflictKey>,
    pub selection_key: Option<Poe2SelectionKey>,
}

#[derive(Debug, Clone)]
pub struct Poe2ModifierDefinition {
    pub id: Poe2ModifierId,
    pub kind: ModType,
    pub required_level: u16,
    pub stats: Vec<Poe2ModifierStat>,
    pub groups: Vec<u64>,
    pub spawn_weights: Vec<HashedTagWeight>,
    pub generation_type: GenerationType,
}

impl ModifierDefinitionIdentity for Poe2ModifierDefinition {
    type Id = Poe2ModifierId;
    fn modifier_definition_id(&self) -> Self::Id {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct Poe2ModifierInstance {
    pub rolls: Vec<i64>,
    pub origin: ModOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Poe2Effect {
    GlobalStat {
        id: u64,
        value: i64,
        mod_type: ModType,
    },
    LocalStat {
        slot: EquipSlot,
        id: u64,
        value: i64,
        mod_type: ModType,
    },
    ScaledStat {
        target_id: u64,
        multiplier: i64,
        scaling: Poe2Scaling,
        mod_type: ModType,
    },
}

impl Game for Poe2 {
    type ItemBase = ItemClass;
    type ItemState = Poe2ItemState;
    type ModifierDefinitionId = Poe2ModifierId;
    type ModifierDefinition = Poe2ModifierDefinition;

    type ModifierInstance = Poe2ModifierInstance;

    type Effect = Poe2Effect;
    type EffectCondition = Poe2Condition;
    type EffectSourceId = String;
}
