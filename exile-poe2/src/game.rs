use exile_core::game::*;

pub struct Poe2;

#[derive(Debug, Default, PartialEq)]
pub struct ItemState {
    item_level: u16,
    dex_requirement: u16,
    int_requirement: u16,
    str_requirement: u16,
}

impl Game for Poe2 {
    type ItemBase;
    type ItemState = ItemState;
    type ModifierDefinitionId;
    type ModifierDefinition;

    type ModifierInstance;

    type Effect;

    type EffectCondition;

    type EffectSourceId;
}
