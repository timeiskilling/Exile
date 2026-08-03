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

#[derive(Debug, Default, PartialEq)]
pub struct ItemState {
    item_level: u16,
    dex_requirement: u16,
    int_requirement: u16,
    str_requirement: u16,
}

impl Game for Poe2 {
    type ItemBase = ItemBase;
    type ItemState = ItemState;
    type ModifierDefinitionId;
    type ModifierDefinition;
    type ModifierInstance;
    type Effect;
    type EffectCondition;
    type EffectSourceId;
}
