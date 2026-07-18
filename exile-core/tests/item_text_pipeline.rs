mod support;

use exile_core::{
    effect::{
        EffectCalculator, EffectCollection, EffectCollectionEvaluator, EffectOrigin,
        ItemEffectCollector,
    },
    item::ItemTextParser,
};

use support::*;

#[test]
fn parses_validates_and_calculates_item_from_text() {
    let text = r#"
        Base: Boots
        Item Level: 86
        +25 to Maximum Life
        20% increased Movement Speed
        Grants Chaos Inoculation
    "#;

    let parser = TestItemTextParser::default();

    let draft = parser.parse(text).expect("item text should be parsed");

    assert_eq!(draft.revision(), 0);
    assert_eq!(draft.modifiers().len(), 3);

    let definition_provider = TestModifierDefinitionProvider::new(vec![
        maximum_life_definition(),
        movement_speed_definition(),
        grants_chaos_inoculation_definition(),
    ]);

    let validator = TestItemValidator::new(&definition_provider);

    let item = draft
        .validate(&validator)
        .expect("parsed item should be valid");

    assert_eq!(item.revision(), 0);
    assert_eq!(item.modifiers().len(), 3);

    let resolver = TestModifierEffectResolver::default();

    let item_collector = ItemEffectCollector::new(&definition_provider, &resolver);

    let mut collection = EffectCollection::<TestGame>::new();

    collection
        .collect_from_item(&item_collector, &item)
        .expect("item effects should be collected");

    assert_eq!(collection.len(), 4);

    let effects = collection
        .iter()
        .map(|entry| entry.effect())
        .collect::<Vec<_>>();

    assert_eq!(effects[0], &TestEffect::AddedMaximumLife { amount: 25 },);

    assert_eq!(
        effects[1],
        &TestEffect::IncreasedMovementSpeed { percent: 20 },
    );

    assert_eq!(effects[2], &TestEffect::ChaosImmune,);

    assert_eq!(effects[3], &TestEffect::SetMaximumLife { value: 1 },);

    let collection_evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active_effects = collection_evaluator
        .collect_active(&collection, &context)
        .expect("effects should be evaluated");

    assert_eq!(active_effects.len(), 4);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );
    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let stats = calculator
        .calculate_from_input(&active_effects, &TestEffectAccumulatorFactory, &input)
        .expect("final stats should be calculated");

    assert_eq!(stats.maximum_life, 1);

    assert!(stats.chaos_immune);

    assert_eq!(stats.increased_movement_speed_percent, 20,);

    assert_eq!(stats.increased_damage_percent, 0,);
}

#[test]
fn conditional_effect_from_text_depends_on_context() {
    let text = r#"
        Base: Boots
        Item Level: 86
        Grants Full Life Damage
    "#;

    let parser = TestItemTextParser::default();

    let draft = parser.parse(text).expect("item text should be parsed");

    assert_eq!(draft.revision(), 0);
    assert_eq!(draft.modifiers().len(), 1);

    let definition_provider =
        TestModifierDefinitionProvider::new(vec![grants_full_life_damage_definition()]);

    let validator = TestItemValidator::new(&definition_provider);

    let item = draft
        .validate(&validator)
        .expect("parsed item should be valid");

    let resolver = TestModifierEffectResolver::default();

    let item_collector = ItemEffectCollector::new(&definition_provider, &resolver);

    let mut collection = EffectCollection::<TestGame>::new();

    collection
        .collect_from_item(&item_collector, &item)
        .expect("item effects should be collected");

    assert_eq!(collection.len(), 1);

    let entry = collection.iter().next().expect("effect entry should exist");

    assert_eq!(entry.effect(), &TestEffect::IncreasedDamage { percent: 20 },);

    assert_eq!(
        entry.condition(),
        Some(&TestEffectCondition::EnemyOnFullLife,),
    );

    let collection_evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let full_life_context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active_at_full_life = collection_evaluator
        .collect_active(&collection, &full_life_context)
        .expect("conditions should be evaluated");

    assert_eq!(active_at_full_life.len(), 1);

    let active_entry = active_at_full_life
        .iter()
        .next()
        .expect("active entry should exist");

    assert!(matches!(
        active_entry.origin(),
        EffectOrigin::ItemModifier {
            definition_id: TestModifierKind::GrantsPassiveNode {
                node_id: TestPassiveNodeId::FullLifeDamage,
            },
            ..
        }
    ));

    let stats_at_full_life = calculator
        .calculate_from_input(&active_at_full_life, &TestEffectAccumulatorFactory, &input)
        .expect("stats should be calculated");

    assert_eq!(stats_at_full_life.increased_damage_percent, 20,);

    let damaged_context = TestEffectContext {
        enemy_current_life: 50,
        enemy_maximum_life: 100,
    };

    let active_when_damaged = collection_evaluator
        .collect_active(&collection, &damaged_context)
        .expect("conditions should be evaluated");

    assert_eq!(active_when_damaged.len(), 0);

    let stats_when_damaged = calculator
        .calculate_from_input(&active_when_damaged, &TestEffectAccumulatorFactory, &input)
        .expect("stats should be calculated");

    assert_eq!(stats_when_damaged.increased_damage_percent, 0,);
}

#[test]
fn parses_item_with_range_modifier() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Base: Boots
        Item Level: 86
        Adds 10 to 20 Physical Damage
    "#;

    let item = parser.parse(text).expect("item should parse");

    assert_eq!(item.modifiers().len(), 1);

    let stored = &item.modifiers()[0];

    assert_eq!(
        stored.definition_id(),
        &TestModifierKind::AddedPhysicalDamage,
    );

    assert_eq!(stored.modifier(), &TestModifier::Range { min: 10, max: 20 },);

    assert_eq!(item.revision(), 0);
}

#[test]
fn calculates_added_physical_damage_from_item_text() {
    let text = r#"
        Base: Boots
        Item Level: 86
        Adds 10 to 20 Physical Damage
    "#;

    let parser = TestItemTextParser::default();

    let draft = parser.parse(text).expect("item text should parse");

    assert_eq!(draft.modifiers().len(), 1);

    let definition_provider =
        TestModifierDefinitionProvider::new(vec![added_physical_damage_definition()]);

    let validator = TestItemValidator::new(&definition_provider);

    let item = draft.validate(&validator).expect("item should be valid");

    let resolver = TestModifierEffectResolver::default();

    let collector = ItemEffectCollector::new(&definition_provider, &resolver);

    let mut collection = EffectCollection::<TestGame>::new();

    collection
        .collect_from_item(&collector, &item)
        .expect("item effects should be collected");

    assert_eq!(collection.len(), 1);

    let entry = collection
        .iter()
        .next()
        .expect("physical damage effect should exist");

    assert_eq!(
        entry.effect(),
        &TestEffect::AddedPhysicalDamage { min: 10, max: 20 },
    );

    assert_eq!(entry.condition(), None);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 50,
        enemy_maximum_life: 100,
    };

    let active_effects = evaluator
        .collect_active(&collection, &context)
        .expect("effects should be evaluated");

    assert_eq!(active_effects.len(), 1);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let stats = calculator
        .calculate_from_input(&active_effects, &TestEffectAccumulatorFactory, &input)
        .expect("stats should be calculated");

    assert_eq!(stats.added_physical_damage_min, 10,);

    assert_eq!(stats.added_physical_damage_max, 20,);
}
