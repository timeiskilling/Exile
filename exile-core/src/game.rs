pub trait ModifierDefinitionIdentity {
    type Id;

    fn modifier_definition_id(&self) -> Self::Id;
}

pub trait Game {
    type ItemBase;
    type ItemState;

    type ModifierDefinitionId;
    type ModifierDefinition: ModifierDefinitionIdentity<Id = Self::ModifierDefinitionId>;

    type ModifierInstance;

    type Effect;
    type EffectCondition;
}
