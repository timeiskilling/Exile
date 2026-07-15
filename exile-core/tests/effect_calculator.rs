mod support;

use std::convert::Infallible;

use exile_core::effect::{
    EffectAccumulatorFactory, EffectAccumulatorFinalizer, EffectApplier, EffectCollection,
    EffectCollectionEvaluator, EffectEntry, EffectSource,
    {EffectCalculationError, EffectCalculationFromInputError, EffectCalculator},
};

use support::{
    effect::{
        TestEffectAccumulator, TestEffectAccumulatorFinalizer, TestEffectApplier,
        TestEffectConditionEvaluator, TestEffectContext, TestEffectFinalizeError, TestPassiveNode,
    },
    game::{TestEffect, TestGame},
};

use crate::support::{TestCalculationInput, TestEffectAccumulatorFactory};

#[test]
fn calculates_final_stats_from_active_effects() {
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
        .expect("condition evaluation should succeed");

    let calculator = EffectCalculator::new(TestEffectApplier, TestEffectAccumulatorFinalizer);

    let accumulator = TestEffectAccumulator::with_base_maximum_life(100);

    let stats = calculator
        .calculate(&active, accumulator)
        .expect("calculation should succeed");

    assert_eq!(stats.maximum_life, 1);
    assert!(stats.chaos_immune);
    assert_eq!(stats.increased_damage_percent, 20);
}

#[derive(Debug, PartialEq, Eq)]
enum TestApplyError {
    SetMaximumLifeRejected,
}

struct FailingEffectApplier;

impl EffectApplier<TestGame> for FailingEffectApplier {
    type Accumulator = ();
    type Error = TestApplyError;

    fn apply_effect(
        &self,
        effect: &TestEffect,
        _accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        match effect {
            TestEffect::SetMaximumLife { .. } => Err(TestApplyError::SetMaximumLifeRejected),

            _ => Ok(()),
        }
    }
}

struct PanicFinalizer;

impl EffectAccumulatorFinalizer for PanicFinalizer {
    type Accumulator = ();
    type Output = ();
    type Error = Infallible;

    fn finalize(&self, _accumulator: Self::Accumulator) -> Result<Self::Output, Self::Error> {
        panic!("finalizer must not run after apply error");
    }
}

#[test]
fn returns_apply_error_and_skips_finalization() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::ChaosInoculation);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let calculator = EffectCalculator::new(FailingEffectApplier, PanicFinalizer);

    let result = calculator.calculate(&active, ());

    assert!(matches!(
        result,
        Err(EffectCalculationError::Apply(
            TestApplyError::SetMaximumLifeRejected
        ))
    ));
}

struct AddedLifeSource;

impl EffectSource<TestGame> for AddedLifeSource {
    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![EffectEntry::unconditional(TestEffect::AddedMaximumLife {
            amount: 1,
        })]
    }
}

#[test]
fn returns_finalize_error_after_successful_application() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&AddedLifeSource);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let calculator = EffectCalculator::new(TestEffectApplier, TestEffectAccumulatorFinalizer);

    let accumulator = TestEffectAccumulator::with_base_maximum_life(u32::MAX);

    let result = calculator.calculate(&active, accumulator);

    assert!(matches!(
        result,
        Err(EffectCalculationError::Finalize(
            TestEffectFinalizeError::MaximumLifeOverflow
        ))
    ));
}

#[test]
fn calculates_final_stats_directly_from_input() {
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
        .expect("condition evaluation should succeed");

    let calculator = EffectCalculator::new(TestEffectApplier, TestEffectAccumulatorFinalizer);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let stats = calculator
        .calculate_from_input(&active, &TestEffectAccumulatorFactory, &input)
        .expect("calculation should succeed");

    assert_eq!(stats.maximum_life, 1);
    assert!(stats.chaos_immune);
    assert_eq!(stats.increased_damage_percent, 20);
}

#[derive(Debug, PartialEq, Eq)]
enum TestAccumulatorCreateError {
    MissingBaseMaximumLife,
}

struct FailingAccumulatorFactory;

impl EffectAccumulatorFactory for FailingAccumulatorFactory {
    type Input = TestCalculationInput;
    type Accumulator = TestEffectAccumulator;
    type Error = TestAccumulatorCreateError;

    fn create(&self, _input: &Self::Input) -> Result<Self::Accumulator, Self::Error> {
        Err(TestAccumulatorCreateError::MissingBaseMaximumLife)
    }
}

#[test]
fn returns_accumulator_creation_error() {
    let collection = EffectCollection::<TestGame>::new();

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let calculator = EffectCalculator::new(TestEffectApplier, TestEffectAccumulatorFinalizer);

    let input = TestCalculationInput {
        base_maximum_life: 0,
    };

    let result = calculator.calculate_from_input(&active, &FailingAccumulatorFactory, &input);

    assert!(matches!(
        result,
        Err(EffectCalculationFromInputError::CreateAccumulator(
            TestAccumulatorCreateError::MissingBaseMaximumLife
        ))
    ));
}
