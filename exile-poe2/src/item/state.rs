use exile_core::game::{Game, ModifierDefinitionIdentity};

use crate::repoe_parse::{GenerationType, ItemClass, Properties, Requirements};

pub struct Poe2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemRarity {
    Normal,
    Magic,
    Rare,
    Unique,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Poe2ModifierId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Poe2ModifierKind(pub String);

#[derive(Debug, Clone)]
pub struct Poe2ModifierStat {
    pub id: String,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone)]
pub struct Poe2ModifierDefinition {
    pub id: Poe2ModifierId,
    pub kind: Poe2ModifierKind,
    pub required_level: u16,
    pub stats: Vec<Poe2ModifierStat>,
    pub groups: Vec<String>,
    pub generation_type: GenerationType,
}

impl ModifierDefinitionIdentity for Poe2ModifierDefinition {
    type Id = Poe2ModifierId;
    fn modifier_definition_id(&self) -> Self::Id {
        self.id.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Poe2Effect {}

impl Game for Poe2 {
    type ItemBase = ItemClass;
    type ItemState = Poe2ItemState;
    type ModifierDefinitionId = Poe2ModifierId;
    type ModifierDefinition = Poe2ModifierDefinition;

    type ModifierInstance = Vec<i64>;

    type Effect = Poe2Effect;
    type EffectCondition = ();
    type EffectSourceId = String;
}
