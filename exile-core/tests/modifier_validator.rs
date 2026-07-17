mod support;

use exile_core::item::{ItemInstance, ModifierValidator};

use support::*;

#[test]
fn accepts_valid_range_modifier() {
    let item = create_valid_item();
    let definition = added_physical_damage_definition();

    let result = TestModifierValidator.validate_modifier(
        &item,
        &definition,
        &TestModifier::Range { min: 10, max: 20 },
    );

    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_reversed_range_modifier() {
    let item = create_valid_item();
    let definition = added_physical_damage_definition();

    let result = TestModifierValidator.validate_modifier(
        &item,
        &definition,
        &TestModifier::Range { min: 20, max: 10 },
    );

    assert_eq!(
        result,
        Err(TestModifierValidationError::InvalidModifierRange),
    );
}

#[test]
fn rejects_range_outside_definition_bounds() {
    let item = create_valid_item();
    let definition = added_physical_damage_definition();

    let result = TestModifierValidator.validate_modifier(
        &item,
        &definition,
        &TestModifier::Range { min: 0, max: 20 },
    );

    assert_eq!(result, Err(TestModifierValidationError::RollOutOfRange),);
}

#[test]
fn rejects_wrong_payload_for_definition() {
    let item = create_valid_item();
    let definition = added_physical_damage_definition();

    let result = TestModifierValidator.validate_modifier(
        &item,
        &definition,
        &TestModifier::Rolled { roll: 15 },
    );

    assert_eq!(
        result,
        Err(TestModifierValidationError::InvalidModifierPayload),
    );
}

#[test]
fn rejects_modifier_when_item_level_is_too_low() {
    let item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 10 },
    );

    let definition = movement_speed_definition();

    let result = TestModifierValidator.validate_modifier(
        &item,
        &definition,
        &TestModifier::Rolled { roll: 20 },
    );

    assert_eq!(result, Err(TestModifierValidationError::ItemLevelTooLow),);
}
