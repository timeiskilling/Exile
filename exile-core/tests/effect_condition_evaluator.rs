
mod support;

use exile_core::effect::effect_condition_evaluator::EffectConditionEvaluator;

use support::{
    effect::{
        TestEffectConditionEvaluator,
        TestEffectContext,
    },
    game::TestEffectCondition,
};

#[test]
fn condition_is_active_when_enemy_is_on_full_life() {
    let evaluator = TestEffectConditionEvaluator;
    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let result = evaluator.evaluate_condition(
        &TestEffectCondition::EnemyOnFullLife,
        &context,
    );

    assert!(matches!(result, Ok(true)));
}

#[test]
fn condition_is_inactive_when_enemy_is_not_on_full_life() {
    let evaluator = TestEffectConditionEvaluator;
    let context = TestEffectContext {
        enemy_current_life: 99,
        enemy_maximum_life: 100,
    };

    let result = evaluator.evaluate_condition(
        &TestEffectCondition::EnemyOnFullLife,
        &context,
    );

    assert!(matches!(result, Ok(false)));
}
