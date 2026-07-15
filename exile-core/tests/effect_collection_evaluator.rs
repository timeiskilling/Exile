mod support;

use exile_core::effect::{EffectCollection, EffectCollectionEvaluator};

use support::{
    effect::{TestEffectConditionEvaluator, TestEffectContext, TestPassiveNode},
    game::{TestEffect, TestGame},
};
#[test]
fn collects_unconditional_and_satisfied_conditional_effects_in_order() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::ChaosInoculation);

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("effect evaluation should succeed");

    assert_eq!(active.len(), 3);

    let effects = active.effects().collect::<Vec<_>>();

    assert_eq!(effects[0], &TestEffect::ChaosImmune,);

    assert_eq!(effects[1], &TestEffect::SetMaximumLife { value: 1 },);

    assert_eq!(effects[2], &TestEffect::IncreasedDamage { percent: 20 },);
}
#[test]
fn excludes_effect_when_condition_is_not_satisfied() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::ChaosInoculation);

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 99,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("effect evaluation should succeed");

    assert_eq!(active.len(), 2);
    assert_eq!(collection.len(), 3);

    let effects = active.effects().collect::<Vec<_>>();

    assert_eq!(effects[0], &TestEffect::ChaosImmune,);

    assert_eq!(effects[1], &TestEffect::SetMaximumLife { value: 1 },);
}

#[test]
fn evaluating_empty_collection_returns_empty_active_collection() {
    let collection = EffectCollection::<TestGame>::new();

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("effect evaluation should succeed");

    assert!(active.is_empty());
    assert_eq!(active.len(), 0);
}

#[test]
fn effects_iterator_does_not_consume_active_collection() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::ChaosInoculation);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("effect evaluation should succeed");

    let effect_count = active.effects().count();

    assert_eq!(effect_count, 2);
    assert_eq!(active.len(), 2);
}
