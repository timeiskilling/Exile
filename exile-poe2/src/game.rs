use exile_core::game::*;

pub struct Poe2;

pub enum DamageType {
    Physical,
    Fire,
    Cold,
    Lightning,
    Chaos,
}

pub enum ItemBase {
    Amulet,
    Armor,
    Belt,
    Boots,
    Bow,
    Buckler,
    Claw,
    Crossbow,
    Dagger,
    FishingRod,
    Flail,
    Focus,
    Gloves,
    Helmet,
    OneHandedAxe,
    OneHandedMace,
    OneHandedSword,
    Quarterstaff,
    Quiver,
    Ring,
    Scepter,
    Shield,
    SkillGem,
    Spear,
    Staff,
    SupportGem,
    Talisman,
    TwoHandedAxe,
    TwoHandedMace,
    TwoHandedSword,
    Wand,
}

#[derive(Debug, PartialEq, Clone, Default)]
struct Requirements {
    strength: u32,
    dexterity: u32,
    intelligence: u32,
    level: u32,
}

#[derive(Debug, Default, PartialEq)]
pub struct ItemState {
    requirements: Requirements,
}

// impl Game for Poe2 {
//     type ItemBase = ItemBase;
//     type ItemState = ItemState;
//     type ModifierDefinitionId;
//     type ModifierDefinition;
//     type ModifierInstance;
//     type Effect;
//     type EffectCondition;
//     type EffectSourceId;
// }
