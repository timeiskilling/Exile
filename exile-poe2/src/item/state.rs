use exile_core::game::{Game, ModifierDefinitionIdentity};

use crate::poe2_condition::Poe2Condition;
use crate::poe2_scaling::Poe2Scaling;
use crate::repoe_parse::{GenerationType, HashedTagWeight, ItemClass, Properties, Requirements};
use ahash::AHasher;
use ahash::HashSet;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Poe2ModifierKind(pub u64);

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
}

#[derive(Debug, Clone)]
pub struct Poe2ModifierDefinition {
    pub id: Poe2ModifierId,
    pub kind: Poe2ModifierKind,
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
    },
    LocalStat {
        slot: EquipSlot,
        id: u64,
        value: i64,
    },
    ScaledStat {
        target_id: u64,
        multiplier: i64,
        scaling: Poe2Scaling,
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
