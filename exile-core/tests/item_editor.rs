mod support;

use exile_core::item::{
    item_editor::ItemEditor, item_instance::ItemInstance, item_validator::ItemValidator,
};
use exile_error::{RemoveModifierError, ReplaceModifierError};

use support::*;

#[test]
fn adds_valid_modifier() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let modifier = TestModifier::Rolled { roll: 27 };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, modifier);

    assert!(result.is_ok());
    assert_eq!(item.modifiers().len(), 1);
    assert_eq!(
        item.modifiers()[0].modifier(),
        &TestModifier::Rolled { roll: 27 }
    );
}

#[test]
fn rejects_modifier_for_non_boots() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: false },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 });

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), TestItemRuleError::NotBoots);
    assert!(item.modifiers().is_empty());
}

#[test]
fn rejects_modifier_when_item_level_is_too_low() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 50 },
    );

    let definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 });

    assert_eq!(result, Err(TestItemRuleError::ItemLevelTooLow),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn rejects_roll_below_minimum() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 19 });

    assert_eq!(result, Err(TestItemRuleError::RollOutOfRange),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn rejects_roll_above_maximum() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let result = editor.add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 31 });

    assert_eq!(result, Err(TestItemRuleError::RollOutOfRange),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn failed_add_does_not_change_existing_modifiers() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .unwrap();

    let result = editor.add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 100 });

    assert_eq!(result, Err(TestItemRuleError::RollOutOfRange),);

    assert_eq!(item.modifiers().len(), 1);
    assert_eq!(
        item.modifiers()[0].modifier(),
        &TestModifier::Rolled { roll: 25 }
    );
}

#[test]
fn added_modifiers_receive_unique_ids() {
    let mut item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
    );

    let definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 20,
        max_roll: 30,
    };

    let editor = ItemEditor::new(TestRules);

    let first_id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .unwrap();

    let second_id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    assert_ne!(first_id, second_id);
}

#[test]
fn removes_modifier_by_id() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    let removed = editor.remove_modifier(&mut item, id).unwrap();

    assert_eq!(removed, TestModifier::Rolled { roll: 27 });
    assert!(item.modifiers().is_empty());
    assert!(item.modifier(id).is_none());
}

#[test]
fn removing_same_modifier_twice_returns_error() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    editor.remove_modifier(&mut item, id).unwrap();

    let result = editor.remove_modifier(&mut item, id);

    assert_eq!(result, Err(RemoveModifierError::ModifierNotFound),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn replaces_modifier_by_id() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .unwrap();

    let previous = editor
        .replace_modifier(
            &mut item,
            id,
            &definition,
            TestModifier::Rolled { roll: 29 },
        )
        .unwrap();

    assert_eq!(previous, TestModifier::Rolled { roll: 25 });

    assert_eq!(item.modifiers().len(), 1);
    assert_eq!(item.modifier(id), Some(&TestModifier::Rolled { roll: 29 }));
}

#[test]
fn failed_replace_does_not_change_modifier() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .unwrap();

    let result = editor.replace_modifier(
        &mut item,
        id,
        &definition,
        TestModifier::Rolled { roll: 100 },
    );

    assert_eq!(
        result,
        Err(ReplaceModifierError::Validation(
            TestItemRuleError::RollOutOfRange,
        )),
    );

    assert_eq!(item.modifiers().len(), 1);
    assert_eq!(item.modifier(id), Some(&TestModifier::Rolled { roll: 25 }));
}

#[test]
fn replacing_removed_modifier_returns_error() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .unwrap();

    editor.remove_modifier(&mut item, id).unwrap();

    let result = editor.replace_modifier(
        &mut item,
        id,
        &definition,
        TestModifier::Rolled { roll: 29 },
    );

    assert_eq!(result, Err(ReplaceModifierError::ModifierNotFound),);

    assert!(item.modifiers().is_empty());
}

#[test]
fn successful_add_increments_revision() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    assert_eq!(item.revision(), 0);

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    assert_eq!(item.revision(), 1);
}

#[test]
fn failed_add_does_not_increment_revision() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let result = editor.add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 100 });

    assert_eq!(result, Err(TestItemRuleError::RollOutOfRange));
    assert_eq!(item.revision(), 0);
}

#[test]
fn successful_remove_increments_revision() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    assert_eq!(item.revision(), 1);

    editor.remove_modifier(&mut item, id).unwrap();

    assert_eq!(item.revision(), 2);
}

#[test]
fn replaces_item_state() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let previous = editor
        .replace_state(&mut item, TestItemState { item_level: 90 })
        .unwrap();

    assert_eq!(previous.item_level, 86);
    assert_eq!(item.state().item_level, 90);
    assert_eq!(item.revision(), 1);
}

#[test]
fn failed_state_replace_does_not_change_item() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let revision_before = item.revision();

    let result = editor.replace_state(&mut item, TestItemState { item_level: 0 });

    assert_eq!(result, Err(TestItemRuleError::InvalidItemLevel));

    assert_eq!(item.state().item_level, 86);
    assert_eq!(item.revision(), revision_before);
}

#[test]
fn validates_valid_item() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    let validator = TestItemValidator;

    let result = validator.validate_item(&item);

    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_item_with_invalid_state() {
    let item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 0 },
    );

    let validator = TestItemValidator;

    let result = validator.validate_item(&item);

    assert_eq!(result, Err(TestItemValidationError::InvalidItemLevel),);
}

#[test]
fn removes_modifier_when_rules_allow_it() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    let removed = editor.remove_modifier(&mut item, id).unwrap();

    assert_eq!(removed, TestModifier::Rolled { roll: 27 });
    assert!(item.modifier(id).is_none());
}

#[test]
fn failed_remove_does_not_change_item() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 30 })
        .unwrap();

    let revision_before = item.revision();

    let result = editor.remove_modifier(&mut item, id);

    assert_eq!(
        result,
        Err(RemoveModifierError::Validation(
            TestItemRuleError::ModifierCannotBeRemoved,
        )),
    );

    assert_eq!(item.modifier(id), Some(&TestModifier::Rolled { roll: 30 }));

    assert_eq!(item.revision(), revision_before,);
}

#[test]
fn item_modifier_full_lifecycle_preserves_invariants() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let validator = TestItemValidator;
    let definition = create_definition();

    assert_eq!(item.revision(), 0);
    assert!(item.modifiers().is_empty());

    let id = editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .unwrap();

    assert_eq!(item.revision(), 1);
    assert_eq!(item.modifier(id), Some(&TestModifier::Rolled { roll: 25 }));

    let previous_modifier = editor
        .replace_modifier(
            &mut item,
            id,
            &definition,
            TestModifier::Rolled { roll: 29 },
        )
        .unwrap();

    assert_eq!(previous_modifier, TestModifier::Rolled { roll: 25 });
    assert_eq!(item.revision(), 2);
    assert_eq!(item.modifier(id), Some(&TestModifier::Rolled { roll: 29 }));
    assert_eq!(item.modifiers().len(), 1);

    let previous_state = editor
        .replace_state(&mut item, TestItemState { item_level: 90 })
        .unwrap();

    assert_eq!(previous_state.item_level, 86);
    assert_eq!(item.state().item_level, 90);
    assert_eq!(item.revision(), 3);

    validator.validate_item(&item).unwrap();

    let removed_modifier = editor.remove_modifier(&mut item, id).unwrap();

    assert_eq!(removed_modifier, TestModifier::Rolled { roll: 29 });
    assert_eq!(item.revision(), 4);
    assert!(item.modifiers().is_empty());
    assert!(item.modifier(id).is_none());

    validator.validate_item(&item).unwrap();
}
