mod support;

use exile_core::{
    effect::{EffectCollection, ItemEffectCollectionError, ItemEffectCollector},
    item::ItemEditor,
};

use support::*;

#[test]
fn collects_effects_from_multiple_sources() {
    let first_node = TestPassiveNode::ChaosInoculation;

    let second_node = TestPassiveNode::FullLifeDamage;

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&first_node);
    collection.collect_from_source(&second_node);

    assert_eq!(collection.len(), 3);

    assert!(
        collection
            .iter()
            .any(|entry| { entry.effect() == &TestEffect::ChaosImmune })
    );

    assert!(
        collection
            .iter()
            .any(|entry| { entry.effect() == &TestEffect::SetMaximumLife { value: 1 } })
    );

    assert!(collection.iter().any(|entry| {
        entry.effect() == &TestEffect::IncreasedDamage { percent: 20 }
            && entry.condition() == Some(&TestEffectCondition::EnemyOnFullLife)
    }));
}

#[test]
fn effect_collection_can_be_consumed() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    let entries: Vec<_> = collection.into_iter().collect();

    assert_eq!(entries.len(), 1);
}

#[test]
fn collects_all_item_effects_after_existing_source_effects() {
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

    let validator = TestItemValidator::new(&provider);

    let item = item.validate(&validator).expect("item should be valid");
    let resolver = TestModifierEffectResolver::default();

    let collector = ItemEffectCollector::new(&provider, &resolver);

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    collection
        .collect_from_item(&collector, &item)
        .expect("item effects should be collected");

    assert_eq!(collection.len(), 3);

    let effects = collection
        .iter()
        .map(|entry| entry.effect())
        .collect::<Vec<_>>();

    assert_eq!(effects[0], &TestEffect::IncreasedDamage { percent: 20 },);

    assert_eq!(effects[1], &TestEffect::AddedMaximumLife { amount: 25 },);

    assert_eq!(
        effects[2],
        &TestEffect::IncreasedMovementSpeed { percent: 20 },
    );
}

#[test]
fn item_collection_error_does_not_change_effect_collection() {
    let mut draft = create_valid_item();
    let editor = ItemEditor::new(TestRules);

    /*
     * Definitions для додавання modifiers у draft.
     */
    let maximum_life_def = maximum_life_definition();

    editor
        .add_modifier(
            &mut draft,
            &maximum_life_def,
            TestModifier::Rolled { roll: 25 },
        )
        .expect("maximum life modifier should be added");

    let movement_speed_def = movement_speed_definition();

    editor
        .add_modifier(
            &mut draft,
            &movement_speed_def,
            TestModifier::Rolled { roll: 20 },
        )
        .expect("movement speed modifier should be added");

    /*
     * Validator повинен мати всі definitions,
     * які використовуються предметом.
     *
     * Helpers викликаємо повторно, тому що попередні
     * values уже належать локальним змінним.
     */
    let validation_provider = TestModifierDefinitionProvider::new(vec![
        maximum_life_definition(),
        movement_speed_definition(),
    ]);

    let validator = TestItemValidator::new(&validation_provider);

    let item = draft.validate(&validator).expect("item should be valid");

    let collection_provider = TestModifierDefinitionProvider::new(vec![maximum_life_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let collector = ItemEffectCollector::new(&collection_provider, &resolver);

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    let original_len = collection.len();

    let result = collection.collect_from_item(&collector, &item);

    assert!(matches!(
        result,
        Err(ItemEffectCollectionError::DefinitionProvider(
            TestModifierDefinitionProviderError::NotFound(TestModifierKind::MovementSpeed)
        ))
    ));

    assert_eq!(collection.len(), original_len);
    assert_eq!(collection.len(), 1);

    let remaining_effect = collection
        .iter()
        .next()
        .expect("original effect should remain")
        .effect();

    assert_eq!(
        remaining_effect,
        &TestEffect::IncreasedDamage { percent: 20 },
    );
}
