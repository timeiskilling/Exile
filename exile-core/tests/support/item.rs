#![allow(dead_code)]

use exile_core::{
    game::Game,
    item::{
        ModifierDefinitionProvider,
        item_instance::{ItemInstance, ModifierInstanceId},
        item_rule::ItemRule,
        item_validator::ItemValidator,
    },
};

use super::game::{
    TestGame, TestItemBase, TestItemState, TestModifier, TestModifierDefinition, TestModifierKind,
};

pub struct TestRules;
pub struct TestItemValidator;

#[derive(Debug, PartialEq)]
pub enum TestItemRuleError {
    NotBoots,
    ItemLevelTooLow,
    RollOutOfRange,
    InvalidModifierPayload,
    InvalidItemLevel,
    ModifierCannotBeRemoved,
}

#[derive(Debug, PartialEq)]
pub enum TestItemValidationError {
    NotBoots,
    InvalidItemLevel,
    RollOutOfRange,
}

impl ItemRule<TestGame> for TestRules {
    type Error = TestItemRuleError;

    fn validate_add_modifier(
        &self,
        item: &ItemInstance<TestGame>,
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

            (TestModifierKind::GrantsChaosInoculation, TestModifier::NoRoll) => {}

            _ => {
                return Err(TestItemRuleError::InvalidModifierPayload);
            }
        }

        Ok(())
    }

    fn validate_replace_modifier(
        &self,
        item: &ItemInstance<TestGame>,
        _target_id: ModifierInstanceId,
        definition: &TestModifierDefinition,
        modifier: &TestModifier,
    ) -> Result<(), Self::Error> {
        self.validate_add_modifier(item, definition, modifier)
    }

    fn validate_replace_state(
        &self,
        _item: &ItemInstance<TestGame>,
        new_state: &<TestGame as Game>::ItemState,
    ) -> Result<(), Self::Error> {
        if new_state.item_level == 0 {
            return Err(TestItemRuleError::InvalidItemLevel);
        }
        Ok(())
    }

    fn validate_remove_modifier(
        &self,
        _item: &ItemInstance<TestGame>,
        _id: ModifierInstanceId,
        modifier: &TestModifier,
    ) -> Result<(), Self::Error> {
        if matches!(modifier, TestModifier::Rolled { roll: 30 }) {
            return Err(TestItemRuleError::ModifierCannotBeRemoved);
        }

        Ok(())
    }
}

impl ItemValidator<TestGame> for TestItemValidator {
    type Error = TestItemValidationError;

    fn validate_item(&self, item: &ItemInstance<TestGame>) -> Result<(), Self::Error> {
        if !item.base().is_boots {
            return Err(TestItemValidationError::NotBoots);
        }

        if item.state().item_level == 0 {
            return Err(TestItemValidationError::InvalidItemLevel);
        }

        for stored in item.modifiers() {
            let modifier = stored.modifier();

            if let TestModifier::Rolled { roll } = modifier
                && (*roll < 20 || *roll > 30)
            {
                return Err(TestItemValidationError::RollOutOfRange);
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

pub fn grants_chaos_inoculation_definition() -> TestModifierDefinition {
    TestModifierDefinition {
        kind: TestModifierKind::GrantsChaosInoculation,
        required_item_level: 1,
        min_roll: 0,
        max_roll: 0,
    }
}
