#![allow(dead_code)]

use exile_core::{
    game::Game,
    item::{
        ItemInstance, ItemRule, ItemValidator, ModifierDefinitionProvider, ModifierInstanceId,
        Unvalidated,
    },
};

use super::game::{
    TestGame, TestItemBase, TestItemState, TestModifier, TestModifierDefinition, TestModifierKind,
    TestPassiveNodeId,
};

pub struct TestRules;

pub struct TestItemValidator<'a> {
    definitions: &'a TestModifierDefinitionProvider,
}

impl<'a> TestItemValidator<'a> {
    pub fn new(definitions: &'a TestModifierDefinitionProvider) -> Self {
        Self { definitions }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestItemRuleError {
    NotBoots,
    ItemLevelTooLow,
    RollOutOfRange,
    InvalidModifierRange,
    InvalidModifierPayload,
    InvalidItemLevel,
    ModifierCannotBeRemoved,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestItemValidationError {
    NotBoots,
    InvalidItemLevel,

    DefinitionProvider(TestModifierDefinitionProviderError),

    ItemLevelTooLow(TestModifierKind),

    RollOutOfRange(TestModifierKind),

    InvalidModifierRange(TestModifierKind),

    InvalidModifierPayload(TestModifierKind),
}

impl ItemRule<TestGame> for TestRules {
    type Error = TestItemRuleError;

    fn validate_add_modifier(
        &self,
        item: &ItemInstance<TestGame, Unvalidated>,
        definition: &TestModifierDefinition,
        modifier: &TestModifier,
    ) -> Result<(), Self::Error> {
        if !item.base().is_boots {
            return Err(TestItemRuleError::NotBoots);
        }

        if item.state().item_level < definition.required_item_level {
            return Err(TestItemRuleError::ItemLevelTooLow);
        }

        match (definition.kind, modifier) {
            (
                TestModifierKind::MovementSpeed
                | TestModifierKind::MaximumLife
                | TestModifierKind::Unsupported,
                TestModifier::Rolled { roll },
            ) => {
                if *roll < definition.min_roll || *roll > definition.max_roll {
                    return Err(TestItemRuleError::RollOutOfRange);
                }
            }

            (TestModifierKind::GrantsPassiveNode { .. }, TestModifier::NoRoll) => {}

            (TestModifierKind::AddedPhysicalDamage, TestModifier::Range { min, max }) => {
                if min > max {
                    return Err(TestItemRuleError::InvalidModifierRange);
                }

                if *min < definition.min_roll
                    || *min > definition.max_roll
                    || *max < definition.min_roll
                    || *max > definition.max_roll
                {
                    return Err(TestItemRuleError::RollOutOfRange);
                }
            }

            _ => {
                return Err(TestItemRuleError::InvalidModifierPayload);
            }
        }

        Ok(())
    }

    fn validate_replace_modifier(
        &self,
        item: &ItemInstance<TestGame, Unvalidated>,
        _target_id: ModifierInstanceId,
        definition: &TestModifierDefinition,
        modifier: &TestModifier,
    ) -> Result<(), Self::Error> {
        self.validate_add_modifier(item, definition, modifier)
    }

    fn validate_replace_state(
        &self,
        _item: &ItemInstance<TestGame, Unvalidated>,
        new_state: &<TestGame as Game>::ItemState,
    ) -> Result<(), Self::Error> {
        if new_state.item_level == 0 {
            return Err(TestItemRuleError::InvalidItemLevel);
        }
        Ok(())
    }

    fn validate_remove_modifier(
        &self,
        _item: &ItemInstance<TestGame, Unvalidated>,
        _id: ModifierInstanceId,
        modifier: &TestModifier,
    ) -> Result<(), Self::Error> {
        if matches!(modifier, TestModifier::Rolled { roll: 30 }) {
            return Err(TestItemRuleError::ModifierCannotBeRemoved);
        }

        Ok(())
    }
}

impl ItemValidator<TestGame> for TestItemValidator<'_> {
    type Error = TestItemValidationError;

    fn validate_item(&self, item: &ItemInstance<TestGame, Unvalidated>) -> Result<(), Self::Error> {
        if !item.base().is_boots {
            return Err(TestItemValidationError::NotBoots);
        }

        if item.state().item_level == 0 {
            return Err(TestItemValidationError::InvalidItemLevel);
        }

        for stored in item.modifiers() {
            let definition = self
                .definitions
                .definition(stored.definition_id())
                .map_err(TestItemValidationError::DefinitionProvider)?;

            if item.state().item_level < definition.required_item_level {
                return Err(TestItemValidationError::ItemLevelTooLow(definition.kind));
            }

            match (definition.kind, stored.modifier()) {
                (
                    TestModifierKind::MovementSpeed
                    | TestModifierKind::MaximumLife
                    | TestModifierKind::Unsupported,
                    TestModifier::Rolled { roll },
                ) => {
                    if *roll < definition.min_roll || *roll > definition.max_roll {
                        return Err(TestItemValidationError::RollOutOfRange(definition.kind));
                    }
                }

                (TestModifierKind::AddedPhysicalDamage, TestModifier::Range { min, max }) => {
                    if *min > *max {
                        return Err(TestItemValidationError::InvalidModifierRange(
                            definition.kind,
                        ));
                    }

                    if *min < definition.min_roll
                        || *min > definition.max_roll
                        || *max < definition.min_roll
                        || *max > definition.max_roll
                    {
                        return Err(TestItemValidationError::RollOutOfRange(definition.kind));
                    }
                }

                (TestModifierKind::GrantsPassiveNode { .. }, TestModifier::NoRoll) => {}

                _ => {
                    return Err(TestItemValidationError::InvalidModifierPayload(
                        definition.kind,
                    ));
                }
            }
        }

        Ok(())
    }
}

pub fn create_valid_item() -> ItemInstance<TestGame> {
    ItemInstance::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    )
}

pub fn create_definition() -> TestModifierDefinition {
    TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestModifierDefinitionProviderError {
    NotFound(TestModifierKind),
}

pub struct TestModifierDefinitionProvider {
    definitions: Vec<TestModifierDefinition>,
}

impl TestModifierDefinitionProvider {
    pub fn new(definitions: Vec<TestModifierDefinition>) -> Self {
        Self { definitions }
    }
}

impl ModifierDefinitionProvider<TestGame> for TestModifierDefinitionProvider {
    type Error = TestModifierDefinitionProviderError;

    fn definition(&self, id: &TestModifierKind) -> Result<&TestModifierDefinition, Self::Error> {
        self.definitions
            .iter()
            .find(|definition| definition.kind == *id)
            .ok_or(TestModifierDefinitionProviderError::NotFound(*id))
    }
}

pub fn movement_speed_definition() -> TestModifierDefinition {
    TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    }
}

pub fn maximum_life_definition() -> TestModifierDefinition {
    TestModifierDefinition {
        kind: TestModifierKind::MaximumLife,
        required_item_level: 1,
        min_roll: 10,
        max_roll: 25,
    }
}

pub fn added_physical_damage_definition() -> TestModifierDefinition {
    TestModifierDefinition {
        kind: TestModifierKind::AddedPhysicalDamage,
        required_item_level: 1,
        min_roll: 1,
        max_roll: 100,
    }
}

pub fn grants_chaos_inoculation_definition() -> TestModifierDefinition {
    grants_passive_node_definition(TestPassiveNodeId::ChaosInoculation)
}

pub fn grants_passive_node_definition(node_id: TestPassiveNodeId) -> TestModifierDefinition {
    TestModifierDefinition {
        kind: TestModifierKind::GrantsPassiveNode { node_id },
        required_item_level: 1,
        min_roll: 0,
        max_roll: 0,
    }
}

pub fn grants_full_life_damage_definition() -> TestModifierDefinition {
    grants_passive_node_definition(TestPassiveNodeId::FullLifeDamage)
}
