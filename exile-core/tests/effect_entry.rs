mod support;

use std::convert::Infallible;

use exile_core::effect::{EffectConditionEvaluator, EffectEntry};

use support::{
    effect::{TestEffectConditionEvaluator, TestEffectContext},
    game::{TestEffect, TestEffectCondition, TestGame},
};

struct PanicConditionEvaluator;

impl EffectConditionEvaluator<TestGame> for PanicConditionEvaluator {
    type Context = ();
    type Error = Infallible;

    fn evaluate_condition(
        &self,
        _condition: &TestEffectCondition,
        _context: &Self::Context,
    ) -> Result<bool, Self::Error> {
        panic!("evaluator must not be called for an unconditional effect");
    }
}

#[test]
fn unconditional_effect_has_no_condition() {
    let entry = EffectEntry::<TestGame>::unconditional(TestEffect::ChaosImmune);

    assert_eq!(entry.effect(), &TestEffect::ChaosImmune,);

    assert!(entry.condition().is_none());
}

#[test]
fn conditional_effect_keeps_its_condition() {
    let entry = EffectEntry::<TestGame>::conditional(
        TestEffect::IncreasedDamage { percent: 20 },
        TestEffectCondition::EnemyOnFullLife,
    );

    assert_eq!(entry.effect(), &TestEffect::IncreasedDamage { percent: 20 },);

    assert_eq!(
        entry.condition(),
        Some(&TestEffectCondition::EnemyOnFullLife,),
    );
}

#[test]
fn unconditional_effect_is_always_active() {
    let entry = EffectEntry::<TestGame>::unconditional(TestEffect::ChaosImmune);

    let result = entry.is_active(&PanicConditionEvaluator, &());

    assert!(matches!(result, Ok(true)));
}

#[test]
fn conditional_effect_is_active_when_condition_is_satisfied() {
    let entry = EffectEntry::<TestGame>::conditional(
        TestEffect::IncreasedDamage { percent: 20 },
        TestEffectCondition::EnemyOnFullLife,
    );

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let result = entry.is_active(&TestEffectConditionEvaluator, &context);

    assert!(matches!(result, Ok(true)));
}

#[test]
fn conditional_effect_is_inactive_when_condition_is_not_satisfied() {
    let entry = EffectEntry::<TestGame>::conditional(
        TestEffect::IncreasedDamage { percent: 20 },
        TestEffectCondition::EnemyOnFullLife,
    );

    let context = TestEffectContext {
        enemy_current_life: 99,
        enemy_maximum_life: 100,
    };

    let result = entry.is_active(&TestEffectConditionEvaluator, &context);

    assert!(matches!(result, Ok(false)));
}

#[derive(Debug, PartialEq, Eq)]
enum TestEntryEvaluationError {
    ContextUnavailable,
}

struct FailingConditionEvaluator;

impl EffectConditionEvaluator<TestGame> for FailingConditionEvaluator {
    type Context = ();
    type Error = TestEntryEvaluationError;

    fn evaluate_condition(
        &self,
        _condition: &TestEffectCondition,
        _context: &Self::Context,
    ) -> Result<bool, Self::Error> {
        Err(TestEntryEvaluationError::ContextUnavailable)
    }
}

#[test]
fn conditional_effect_propagates_evaluator_error() {
    let entry = EffectEntry::<TestGame>::conditional(
        TestEffect::IncreasedDamage { percent: 20 },
        TestEffectCondition::EnemyOnFullLife,
    );

    let result = entry.is_active(&FailingConditionEvaluator, &());

    assert!(matches!(
        result,
        Err(TestEntryEvaluationError::ContextUnavailable)
    ));
}
