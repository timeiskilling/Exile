mod support;

use exile_core::{
    effect::{EffectCalculator, EffectCollection, EffectCollectionEvaluator, ItemEffectCollector},
    item::ItemEditor,
};

use support::{
    effect::{
        TestCalculationInput, TestEffectAccumulatorFactory, TestEffectAccumulatorFinalizer,
        TestEffectApplier, TestEffectConditionEvaluator, TestEffectContext,
        TestModifierEffectResolver, TestPassiveNode,
    },
    game::{TestGame, TestModifier, TestModifierKind},
};

use crate::support::{
    TestEffect, TestItemValidator, TestModifierDefinitionProvider, TestRules, create_definition,
    create_valid_item, grants_chaos_inoculation_definition, maximum_life_definition,
};

fn build_effect_collection() -> EffectCollection<TestGame> {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    let resolver = TestModifierEffectResolver::default();

    let mut maximum_life_definition = create_definition();
    maximum_life_definition.kind = TestModifierKind::MaximumLife;

    let maximum_life_modifier = TestModifier::Rolled { roll: 25 };

    collection
        .collect_from_modifier(&resolver, &maximum_life_definition, &maximum_life_modifier)
        .expect("maximum life modifier should resolve");

    let mut movement_speed_definition = create_definition();
    movement_speed_definition.kind = TestModifierKind::MovementSpeed;

    let movement_speed_modifier = TestModifier::Rolled { roll: 15 };

    collection
        .collect_from_modifier(
            &resolver,
            &movement_speed_definition,
            &movement_speed_modifier,
        )
        .expect("movement speed modifier should resolve");

    collection
}

#[test]
fn calculates_final_stats_from_passive_node_and_modifiers() {
    let collection = build_effect_collection();

    assert_eq!(collection.len(), 3);

    let condition_evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = condition_evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    assert_eq!(active.len(), 3);

    let calculator = EffectCalculator::new(TestEffectApplier, TestEffectAccumulatorFinalizer);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let stats = calculator
        .calculate_from_input(&active, &TestEffectAccumulatorFactory, &input)
        .expect("effect calculation should succeed");

    assert_eq!(stats.maximum_life, 125);
    assert_eq!(stats.increased_damage_percent, 20);

    assert_eq!(stats.increased_movement_speed_percent, 15,);

    assert!(!stats.chaos_immune);
}

#[test]
fn calculates_final_stats_from_item_modifiers_and_granted_node() {
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

    let grants_chaos_inoculation_definition = grants_chaos_inoculation_definition();

    editor
        .add_modifier(
            &mut item,
            &grants_chaos_inoculation_definition,
            TestModifier::NoRoll,
        )
        .expect("grants Chaos Inoculation modifier should be added");

    assert_eq!(item.modifiers().len(), 2);

    let provider = TestModifierDefinitionProvider::new(vec![
        maximum_life_definition,
        grants_chaos_inoculation_definition,
    ]);

    let validator = TestItemValidator::new(&provider);
    let item = item.validate(&validator).expect("item should be valid");

    let resolver = TestModifierEffectResolver::default();

    let item_collector = ItemEffectCollector::new(&provider, &resolver);

    let mut collection = EffectCollection::<TestGame>::new();

    collection
        .collect_from_item(&item_collector, &item)
        .expect("item effects should be collected");

    assert_eq!(collection.len(), 3);

    let effects = collection
        .iter()
        .map(|entry| entry.effect())
        .collect::<Vec<_>>();

    assert_eq!(effects[0], &TestEffect::AddedMaximumLife { amount: 25 },);

    assert_eq!(effects[1], &TestEffect::ChaosImmune,);

    assert_eq!(effects[2], &TestEffect::SetMaximumLife { value: 1 },);
    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("effect evaluation should succeed");

    assert_eq!(active.len(), 3);
    let calculator = EffectCalculator::new(TestEffectApplier, TestEffectAccumulatorFinalizer);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };
    let stats = calculator
        .calculate_from_input(&active, &TestEffectAccumulatorFactory, &input)
        .expect("effect calculation should succeed");
    assert_eq!(stats.maximum_life, 1);
    assert!(stats.chaos_immune);
    assert_eq!(stats.increased_damage_percent, 0);

    assert_eq!(stats.increased_movement_speed_percent, 0,);
}
