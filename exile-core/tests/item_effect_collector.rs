mod support;

use exile_core::{
    effect::{ItemEffectCollectionError, ItemEffectCollector},
    item::ItemEditor,
};

use support::{
    TestEffect, TestEffectResolveError, TestModifier, TestModifierDefinition,
    TestModifierDefinitionProvider, TestModifierDefinitionProviderError,
    TestModifierEffectResolver, TestModifierKind, TestRules, create_valid_item,
    maximum_life_definition, movement_speed_definition,
};

use crate::support::{TestItemValidator, grants_chaos_inoculation_definition};

#[test]
fn collects_effects_from_all_item_modifiers_in_order() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let maximum_life_definition = maximum_life_definition();

    editor
        .add_modifier(
            &mut item,
            &maximum_life_definition,
            TestModifier::Rolled { roll: 25 },
        )
        .expect("maximum life modifier should be added");

    let movement_speed_definition = movement_speed_definition();

    editor
        .add_modifier(
            &mut item,
            &movement_speed_definition,
            TestModifier::Rolled { roll: 20 },
        )
        .expect("movement speed modifier should be added");

    let provider = TestModifierDefinitionProvider::new(vec![
        maximum_life_definition,
        movement_speed_definition,
    ]);

    let resolver = TestModifierEffectResolver::default();

    let validator = TestItemValidator::new(&provider);
    let item = item.validate(&validator).expect("item should be valid");

    let collector = ItemEffectCollector::new(&provider, &resolver);

    let entries = collector
        .collect(&item)
        .expect("item effects should be collected");

    assert_eq!(entries.len(), 2);

    assert_eq!(
        entries[0].effect(),
        &TestEffect::AddedMaximumLife { amount: 25 },
    );

    assert_eq!(
        entries[1].effect(),
        &TestEffect::IncreasedMovementSpeed { percent: 20 },
    );
}

#[test]
fn returns_definition_provider_error_when_definition_is_missing() {
    let mut draft = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let maximum_life_def = maximum_life_definition();

    editor
        .add_modifier(
            &mut draft,
            &maximum_life_def,
            TestModifier::Rolled { roll: 25 },
        )
        .expect("maximum life modifier should be added");

    let movement_speed_definition = movement_speed_definition();

    editor
        .add_modifier(
            &mut draft,
            &movement_speed_definition,
            TestModifier::Rolled { roll: 20 },
        )
        .expect("movement speed modifier should be added");

    let validation_provider =
        TestModifierDefinitionProvider::new(vec![maximum_life_def, movement_speed_definition]);

    let validator = TestItemValidator::new(&validation_provider);

    let item = draft.validate(&validator).expect("item should be valid");

    let collection_provider = TestModifierDefinitionProvider::new(vec![maximum_life_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let collector = ItemEffectCollector::new(&collection_provider, &resolver);

    let result = collector.collect(&item);

    assert!(matches!(
        result,
        Err(ItemEffectCollectionError::DefinitionProvider(
            TestModifierDefinitionProviderError::NotFound(TestModifierKind::MovementSpeed)
        ))
    ));
}

#[test]
fn returns_resolver_error_for_unsupported_modifier() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let unsupported_definition = TestModifierDefinition {
        kind: TestModifierKind::Unsupported,
        required_item_level: 1,
        min_roll: 10,
        max_roll: 20,
    };

    editor
        .add_modifier(
            &mut item,
            &unsupported_definition,
            TestModifier::Rolled { roll: 15 },
        )
        .expect("item rules should allow the test modifier");

    let provider = TestModifierDefinitionProvider::new(vec![unsupported_definition]);

    let resolver = TestModifierEffectResolver::default();
    let validator = TestItemValidator::new(&provider);
    let item = item.validate(&validator).expect("item should be valid");
    let collector = ItemEffectCollector::new(&provider, &resolver);

    let result = collector.collect(&item);

    assert!(matches!(
        result,
        Err(ItemEffectCollectionError::Resolver(
            TestEffectResolveError::UnsupportedModifier
        ))
    ));
}

#[test]
fn returns_empty_effects_for_item_without_modifiers() {
    let item = create_valid_item();

    let provider = TestModifierDefinitionProvider::new(Vec::new());

    let resolver = TestModifierEffectResolver::default();
    let collector = ItemEffectCollector::new(&provider, &resolver);

    let validator = TestItemValidator::new(&provider);
    let item = item.validate(&validator).expect("item should be valid");

    let entries = collector
        .collect(&item)
        .expect("empty item collection should succeed");

    assert!(entries.is_empty());
}

#[test]
fn collects_effects_from_granted_passive_node() {
    let mut item = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    let definition = grants_chaos_inoculation_definition();

    editor
        .add_modifier(&mut item, &definition, TestModifier::NoRoll)
        .expect("granted node modifier should be added");

    let provider = TestModifierDefinitionProvider::new(vec![definition]);

    let resolver = TestModifierEffectResolver::default();
    let validator = TestItemValidator::new(&provider);
    let item = item.validate(&validator).expect("item should be valid");
    let collector = ItemEffectCollector::new(&provider, &resolver);

    let entries = collector
        .collect(&item)
        .expect("item effects should be collected");

    assert_eq!(entries.len(), 2);

    assert_eq!(entries[0].effect(), &TestEffect::ChaosImmune,);

    assert_eq!(
        entries[1].effect(),
        &TestEffect::SetMaximumLife { value: 1 },
    );
}
