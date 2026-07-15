mod support;

use exile_core::item::{
    item_editor::ItemEditor, item_instance::ItemInstance, item_validator::ItemValidator,
};

use support::*;

#[test]
fn validates_valid_item() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);
    let definition = create_definition();

    editor
        .add_modifier(&mut item, &definition, TestModifier::Rolled { roll: 27 })
        .unwrap();

    let validator = TestItemValidator;

    assert_eq!(validator.validate_item(&item), Ok(()),);
}

#[test]
fn rejects_item_with_invalid_state() {
    let item = ItemInstance::<TestGame>::new(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 0 },
    );

    let validator = TestItemValidator;

    assert_eq!(
        validator.validate_item(&item),
        Err(TestItemValidationError::InvalidItemLevel,),
    );
}
