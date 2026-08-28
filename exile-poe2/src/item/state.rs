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
    Armour,
    ArmourPercent,
    Evasion,
    EvasionPercent,
    Block,
    FireResistance,
    ColdResistance,
    LightningResistance,
    ChaosResistance,
    Strength,
    Dexterity,
    Intelligence,
    ChaosDamagePercent,
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
