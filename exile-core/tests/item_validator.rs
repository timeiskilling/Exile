mod support;

use exile_core::item::{ItemEditor, ItemInstance, ItemValidator};

use support::*;

#[test]
fn validates_valid_item() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    let provider = TestModifierDefinitionProvider::new(vec![definition]);

    let validator = TestItemValidator::new(&provider);

    assert_eq!(validator.validate_item(&item), Ok(()),);
}

#[test]
fn rejects_item_with_invalid_state() {
    let item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 0 },
    );

    let provider = TestModifierDefinitionProvider::new(Vec::new());

    let validator = TestItemValidator::new(&provider);

    assert_eq!(
        validator.validate_item(&item),
        Err(TestItemValidationError::InvalidItemLevel),
    );
}

#[test]
fn validates_modifier_using_its_definition() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let definition = maximum_life_definition();

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .expect("modifier should be added");

    let provider = TestModifierDefinitionProvider::new(vec![definition]);

    let validator = TestItemValidator::new(&provider);

    assert_eq!(validator.validate_item(&item), Ok(()),);
}

#[test]
fn returns_error_when_modifier_definition_is_missing() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let definition = maximum_life_definition();

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 25 })
        .expect("modifier should be added");

    let provider = TestModifierDefinitionProvider::new(Vec::new());

    let validator = TestItemValidator::new(&provider);

    let result = validator.validate_item(&item);

    assert!(matches!(
        result,
        Err(TestItemValidationError::DefinitionProvider(
            TestModifierDefinitionProviderError::NotFound(TestModifierKind::MaximumLife)
        ))
    ));
}

#[test]
fn returns_error_when_item_level_is_too_low_for_stored_modifier() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let definition = movement_speed_definition();

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 20 })
        .expect("modifier should be added");

    /*
     * TestRules для replace_state відхиляє лише 0,
     * тому рівень 10 можна встановити.
     */
    editor
        .replace_state(&mut item, TestItemState { item_level: 10 })
        .expect("state should be replaced");

    let provider = TestModifierDefinitionProvider::new(vec![definition]);

    let validator = TestItemValidator::new(&provider);

    let result = validator.validate_item(&item);

    assert!(matches!(
        result,
        Err(TestItemValidationError::ItemLevelTooLow(
            TestModifierKind::MovementSpeed
        ))
    ));
}

#[test]
fn returns_error_when_roll_is_outside_current_definition_range() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let original_definition = movement_speed_definition();

    editor
        .add_modifier(
            &mut item,
            &original_definition,
            TestModifier::Rolled { roll: 25 },
        )
        .expect("modifier should be added");

    let current_definition = TestModifierDefinition {
        kind: TestModifierKind::MovementSpeed,
        required_item_level: 75,
        min_roll: 26,
        max_roll: 30,
    };

    let provider = TestModifierDefinitionProvider::new(vec![current_definition]);

    let validator = TestItemValidator::new(&provider);

    let result = validator.validate_item(&item);

    assert!(matches!(
        result,
        Err(TestItemValidationError::RollOutOfRange(
            TestModifierKind::MovementSpeed
        ))
    ));
}

#[test]
fn validates_modifier_with_numeric_range() {
    let mut draft = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let definition = added_physical_damage_definition();

    editor
        .add_modifier(
            &mut draft,
            &definition,
            TestModifier::Range { min: 10, max: 20 },
        )
        .expect("range modifier should be added");

    let provider = TestModifierDefinitionProvider::new(vec![definition]);

    let validator = TestItemValidator::new(&provider);

    let item = draft
        .validate(&validator)
        .expect("range modifier should be valid");

    assert_eq!(item.modifiers().len(), 1);
}

#[test]
fn rejects_modifier_with_reversed_range() {
    let draft = ItemInstance::<TestGame>::from_parts(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
        vec![(
            TestModifierKind::AddedPhysicalDamage,
            TestModifier::Range { min: 20, max: 10 },
        )],
    );

    let provider = TestModifierDefinitionProvider::new(vec![added_physical_damage_definition()]);

    let validator = TestItemValidator::new(&provider);

    let result = validator.validate_item(&draft);

    assert!(matches!(
        result,
        Err(TestItemValidationError::InvalidModifierRange(
            TestModifierKind::AddedPhysicalDamage
        ))
    ));
}

#[test]
fn rejects_single_roll_for_range_modifier() {
    let draft = ItemInstance::<TestGame>::from_parts(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
        vec![(
            TestModifierKind::AddedPhysicalDamage,
            TestModifier::Rolled { roll: 15 },
        )],
    );

    let provider = TestModifierDefinitionProvider::new(vec![added_physical_damage_definition()]);

    let validator = TestItemValidator::new(&provider);

    let result = validator.validate_item(&draft);

    assert!(matches!(
        result,
        Err(TestItemValidationError::InvalidModifierPayload(
            TestModifierKind::AddedPhysicalDamage
        ))
    ));
}

#[test]
fn failed_validation_preserves_unvalidated_item() {
    let draft = ItemInstance::<TestGame>::from_parts(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 10 },
        vec![(
            TestModifierKind::MovementSpeed,
            TestModifier::Rolled { roll: 20 },
        )],
    );

    let provider = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let validator = TestItemValidator::new(&provider);

    let failure = match draft.validate(&validator) {
        Ok(_) => panic!("validation should fail"),
        Err(failure) => failure,
    };

    assert_eq!(
        failure.error(),
        &TestItemValidationError::ItemLevelTooLow(TestModifierKind::MovementSpeed,),
    );

    assert_eq!(failure.item().state().item_level, 10);
    assert_eq!(failure.item().modifiers().len(), 1);
    assert_eq!(failure.item().revision(), 0);

    let stored = &failure.item().modifiers()[0];

    assert_eq!(stored.definition_id(), &TestModifierKind::MovementSpeed,);

    assert_eq!(stored.modifier(), &TestModifier::Rolled { roll: 20 },);
}

#[test]
fn preserved_item_can_be_fixed_and_validated_again() {
    let draft = ItemInstance::<TestGame>::from_parts(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 10 },
        vec![(
            TestModifierKind::MovementSpeed,
            TestModifier::Rolled { roll: 20 },
        )],
    );

    let provider = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let validator = TestItemValidator::new(&provider);

    let failure = match draft.validate(&validator) {
        Ok(_) => panic!("validation should fail"),
        Err(failure) => failure,
    };

    assert_eq!(
        failure.error(),
        &TestItemValidationError::ItemLevelTooLow(TestModifierKind::MovementSpeed,),
    );

    let mut draft = failure.into_item();

    let editor = ItemEditor::new(TestRules);

    editor
        .replace_state(&mut draft, TestItemState { item_level: 86 })
        .expect("item level should be replaced");

    assert_eq!(draft.state().item_level, 86);
    assert_eq!(draft.revision(), 1);

    let item = draft
        .validate(&validator)
        .expect("fixed item should be valid");

    assert_eq!(item.state().item_level, 86);
    assert_eq!(item.modifiers().len(), 1);
    assert_eq!(item.revision(), 1);
}
