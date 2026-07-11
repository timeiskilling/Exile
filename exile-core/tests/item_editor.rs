use exile_core::item::{
    game_definition::Game, item_editor::ItemEditor, item_instance::ItemInstance,
    item_rule::ItemRule,
};
use exile_error::RemoveModifierError;

struct TestGame;

struct TestModifierDefinition {
    required_item_level: u16,
    min_roll: u16,
    max_roll: u16,
}

struct TestItemBase {
    is_boots: bool,
}

struct TestItemState {
    item_level: u16,
}

#[derive(Debug, PartialEq)]
struct TestModifier {
    roll: u16,
}

impl Game for TestGame {
    type ItemBase = TestItemBase;
    type ItemState = TestItemState;

    type ModifierDefinition = TestModifierDefinition;
    type ModifierInstance = TestModifier;
}

struct TestRules;

#[derive(Debug, PartialEq)]
enum TestError {
    NotBoots,
    ItemLevelTooLow,
    RollOutOfRange,
}

impl ItemRule<TestGame> for TestRules {
    type Error = TestError;

    fn validate_add_modifier(
        &self,
        item: &ItemInstance<TestGame>,
        definition: &TestModifierDefinition,
        modifier: &TestModifier,
    ) -> Result<(), Self::Error> {
        if !item.base().is_boots {
            return Err(TestError::NotBoots);
        }

        if item.state().item_level < definition.required_item_level {
            return Err(TestError::ItemLevelTooLow);
        }

        if modifier.roll < definition.min_roll || modifier.roll > definition.max_roll {
            return Err(TestError::RollOutOfRange);
        }

        Ok(())
    }
}

#[test]
fn adds_valid_modifier() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let modifier = TestModifier { roll: 27 };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, modifier);

    assert!(result.is_ok());
    assert_eq!(item.modifiers().len(), 1);
    assert_eq!(item.modifiers()[0].modifier().roll, 27);
}

#[test]
fn rejects_modifier_for_non_boots() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: false },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier { roll: 27 });

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), TestError::NotBoots);
    assert!(item.modifiers().is_empty());
}

#[test]
fn rejects_modifier_when_item_level_is_too_low() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 50 },
    );

    let definition = TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier { roll: 27 });

    assert_eq!(result, Err(TestError::ItemLevelTooLow),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn rejects_roll_below_minimum() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier { roll: 19 });

    assert_eq!(result, Err(TestError::RollOutOfRange),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn rejects_roll_above_maximum() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier { roll: 31 });

    assert_eq!(result, Err(TestError::RollOutOfRange),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn failed_add_does_not_change_existing_modifiers() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    editor
        .add_modifier(&mut item, &definition, TestModifier { roll: 25 })
        .unwrap();

    let result = editor.add_modifier(&mut item, &definition, TestModifier { roll: 100 });

    assert_eq!(result, Err(TestError::RollOutOfRange),);

    assert_eq!(item.modifiers().len(), 1);
    assert_eq!(item.modifiers()[0].modifier().roll, 25);
}

#[test]
fn added_modifiers_receive_unique_ids() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let first_id = editor
        .add_modifier(&mut item, &definition, TestModifier { roll: 25 })
        .unwrap();

    let second_id = editor
        .add_modifier(&mut item, &definition, TestModifier { roll: 27 })
        .unwrap();

    assert_ne!(first_id, second_id);
}

#[test]
fn removes_modifier_by_id() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier { roll: 27 })
        .unwrap();

    let removed = editor.remove_modifier(&mut item, id).unwrap();

    assert_eq!(removed.roll, 27);
    assert!(item.modifiers().is_empty());
    assert!(item.modifier(id).is_none());
}

#[test]
fn removing_same_modifier_twice_returns_error() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier { roll: 27 })
        .unwrap();

    editor.remove_modifier(&mut item, id).unwrap();

    let result = editor.remove_modifier(&mut item, id);

    assert_eq!(result, Err(RemoveModifierError::ModifierNotFound),);

    assert!(item.modifiers().is_empty());
}

fn create_valid_item() -> ItemInstance<TestGame> {
    ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    )
}

fn create_definition() -> TestModifierDefinition {
    TestModifierDefinition {
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    }
}
