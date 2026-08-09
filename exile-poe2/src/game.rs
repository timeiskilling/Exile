use exile_core::game::*;

use crate::repoe_parse::{GenerationType, ItemClass, Properties};

pub struct Poe2;

pub enum DamageType {
    Physical,
    Fire,
    Cold,
    Lightning,
    Chaos,
}

#[derive(Debug, PartialEq, Clone, Default)]
struct Requirements {
    strength: u32,
    dexterity: u32,
    intelligence: u32,
    level: u32,
}

#[derive(Debug, Default)]
pub struct ItemState {
    drop_level: u32,
    name: String,
    item_base: ItemClass,
    properties: Properties,
    requirements: Requirements,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Poe2ModifierId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Poe2ModifierKind(String);

pub struct Poe2ModifierStat {
    pub id: String,
    pub min: i64,
    pub max: i64,
}

pub struct Poe2ModifierDefinition {
    pub id: Poe2ModifierId,
    pub kind: Poe2ModifierKind,
    pub required_level: u32,
    pub stats: Vec<Poe2ModifierStat>,
    pub groups: Vec<String>,
    pub implicit_tags: Vec<String>,
    pub generation_type: GenerationType,
}

impl ModifierDefinitionIdentity for Poe2ModifierDefinition {
    type Id = Poe2ModifierId;

    fn modifier_definition_id(&self) -> Self::Id {
        self.id.clone()
    }
}

impl Game for Poe2 {
    type ItemBase = ItemClass;
    type ItemState = ItemState;
    type ModifierDefinitionId = Poe2ModifierId;
    type ModifierDefinition = Poe2ModifierDefinition;

    type ModifierInstance;
    type Effect;
    type EffectCondition;
    type EffectSourceId;
}
