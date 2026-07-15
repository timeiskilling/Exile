mod support;

use exile_core::item::ModifierDefinitionProvider;

use support::{
    game::{TestModifierDefinition, TestModifierKind},
    item::{TestModifierDefinitionProvider, TestModifierDefinitionProviderError},
};


#[test]
fn returns_definition_by_id() {
    let provider = TestModifierDefinitionProvider::new(vec![
        movement_speed_definition(),
        maximum_life_definition(),
    ]);

    let definition = provider
        .definition(&TestModifierKind::MaximumLife)
        .expect("maximum life definition should exist");

    assert_eq!(definition.kind, TestModifierKind::MaximumLife,);

    assert_eq!(definition.required_item_level, 1);
    assert_eq!(definition.min_roll, 10);
    assert_eq!(definition.max_roll, 25);
}

#[test]
fn returns_correct_definition_when_multiple_exist() {
    let provider = TestModifierDefinitionProvider::new(vec![
        maximum_life_definition(),
        movement_speed_definition(),
    ]);

    let definition = provider
        .definition(&TestModifierKind::MovementSpeed)
        .expect("movement speed definition should exist");

    assert_eq!(definition.kind, TestModifierKind::MovementSpeed,);

    assert_eq!(definition.required_item_level, 75);
    assert_eq!(definition.min_roll, 20);
    assert_eq!(definition.max_roll, 30);
}

#[test]
fn returns_error_when_definition_does_not_exist() {
    let provider = TestModifierDefinitionProvider::new(vec![maximum_life_definition()]);

    let result = provider.definition(&TestModifierKind::Unsupported);

    assert!(matches!(
        result,
        Err(TestModifierDefinitionProviderError::NotFound(
            TestModifierKind::Unsupported
        ))
    ));
}
