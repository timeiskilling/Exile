mod support;

use std::convert::Infallible;

use exile_core::effect::{
    ActiveEffectCollection, EffectAccumulatorFactory, EffectAccumulatorFinalizer, EffectApplier,
    EffectCalculationError, EffectCalculationFromInputError, EffectCalculator, EffectCollection,
    EffectCollectionEvaluator, EffectEntry, EffectExecutionPlanValidationError, EffectOrigin,
    EffectSource,
};

use support::{
    effect::{
        TestEffectAccumulator, TestEffectAccumulatorFinalizer, TestEffectApplier,
        TestEffectConditionEvaluator, TestEffectContext, TestEffectFinalizeError, TestPassiveNode,
    },
    game::{TestEffect, TestGame},
};

use crate::support::{
    TestCalculationInput, TestEffectAccumulatorFactory, TestEffectConflictKey, TestEffectSourceId,
    test_effect_execution_planner,
};

struct ConflictingMaximumLifeSource;

impl EffectSource<TestGame> for ConflictingMaximumLifeSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic("conflicting_maximum_life_source")
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![
            EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 1 }),
            EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 10 }),
        ]
    }
}

struct PanicEffectApplier;

impl EffectApplier<TestGame> for PanicEffectApplier {
    type Accumulator = ();
    type Error = Infallible;

    fn apply_effect(
        &self,
        _effect: &TestEffect,
        _accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        panic!("effect applier must not run after plan error");
    }
}

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

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );
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

    let calculator = EffectCalculator::new(
        FailingEffectApplier,
        PanicFinalizer,
        test_effect_execution_planner(),
    );

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

    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic("added_life_source")
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

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );
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

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );
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

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );
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

struct UnorderedCalculationSource;

impl EffectSource<TestGame> for UnorderedCalculationSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic("unordered_calculation_source")
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![
            EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 1 }),
            EffectEntry::unconditional(TestEffect::IncreasedDamage { percent: 20 }),
            EffectEntry::unconditional(TestEffect::AddedMaximumLife { amount: 25 }),
        ]
    }
}

#[derive(Default)]
struct RecordingOrderAccumulator {
    applied: Vec<&'static str>,
}

struct RecordingOrderApplier;

impl EffectApplier<TestGame> for RecordingOrderApplier {
    type Accumulator = RecordingOrderAccumulator;

    type Error = Infallible;

    fn apply_effect(
        &self,
        effect: &TestEffect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        let name = match effect {
            TestEffect::AddedMaximumLife { .. } => "added",

            TestEffect::IncreasedDamage { .. } => "increased",

            TestEffect::SetMaximumLife { .. } => "final",

            _ => "other",
        };

        accumulator.applied.push(name);

        Ok(())
    }
}

struct RecordingOrderFinalizer;

impl EffectAccumulatorFinalizer for RecordingOrderFinalizer {
    type Accumulator = RecordingOrderAccumulator;

    type Output = Vec<&'static str>;

    type Error = Infallible;

    fn finalize(&self, accumulator: Self::Accumulator) -> Result<Self::Output, Self::Error> {
        Ok(accumulator.applied)
    }
}

#[test]
fn calculator_applies_effects_in_phase_order() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&UnorderedCalculationSource);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let calculator = EffectCalculator::new(
        RecordingOrderApplier,
        RecordingOrderFinalizer,
        test_effect_execution_planner(),
    );

    let applied = calculator
        .calculate(&active, RecordingOrderAccumulator::default())
        .expect("calculation should succeed");

    assert_eq!(applied, vec!["added", "increased", "final",],);
}

#[test]
fn returns_plan_error_and_skips_application_and_finalization() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&ConflictingMaximumLifeSource);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let calculator = EffectCalculator::new(
        PanicEffectApplier,
        PanicFinalizer,
        test_effect_execution_planner(),
    );

    let result = calculator.calculate(&active, ());

    assert!(matches!(
        result,
        Err(EffectCalculationError::Plan(
            EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                key: TestEffectConflictKey::MaximumLifeOverride,
                ..
            }
        ))
    ));
}

struct SingleEffectSource {
    id: &'static str,
    effect: TestEffect,
}

impl SingleEffectSource {
    fn new(id: &'static str, effect: TestEffect) -> Self {
        Self { id, effect }
    }
}

impl EffectSource<TestGame> for SingleEffectSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic(self.id)
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![EffectEntry::unconditional(self.effect)]
    }
}

fn collect_active<'a>(
    collection: &'a EffectCollection<TestGame>,
) -> ActiveEffectCollection<'a, TestGame> {
    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    evaluator
        .collect_active(collection, &context)
        .expect("condition evaluation should succeed")
}

fn assert_synthetic_origin(origin: &EffectOrigin<TestGame>, expected: &'static str) {
    match origin {
        EffectOrigin::Source(TestEffectSourceId::Synthetic(actual)) => {
            assert_eq!(*actual, expected);
        }

        _ => {
            panic!("expected synthetic source origin, got {origin:?}");
        }
    }
}

#[test]
fn detailed_calculation_returns_output_plan_and_selection_rejections() {
    let weak = SingleEffectSource::new("weak", TestEffect::MinimumMovementSpeed { percent: 20 });

    let strong =
        SingleEffectSource::new("strong", TestEffect::MinimumMovementSpeed { percent: 30 });

    let medium =
        SingleEffectSource::new("medium", TestEffect::MinimumMovementSpeed { percent: 25 });

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&weak);
    collection.collect_from_source(&strong);
    collection.collect_from_source(&medium);

    let active = collect_active(&collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let calculation = calculator
        .calculate_detailed(&active, TestEffectAccumulator::with_base_maximum_life(100))
        .expect("detailed calculation should succeed");

    assert_eq!(calculation.output().minimum_movement_speed_percent, 30,);

    let plan = calculation.execution_plan();

    assert_eq!(plan.len(), 1);

    let effects = plan.effects().collect::<Vec<_>>();

    assert_eq!(
        effects,
        vec![&TestEffect::MinimumMovementSpeed { percent: 30 },],
    );

    let rejections = plan.selection_rejections().collect::<Vec<_>>();

    assert_eq!(rejections.len(), 2);

    assert_eq!(
        rejections[0].rejected().effect(),
        &TestEffect::MinimumMovementSpeed { percent: 20 },
    );

    assert_eq!(
        rejections[0].winner().effect(),
        &TestEffect::MinimumMovementSpeed { percent: 30 },
    );

    assert_synthetic_origin(rejections[0].rejected().origin(), "weak");

    assert_synthetic_origin(rejections[0].winner().origin(), "strong");

    assert_eq!(
        rejections[1].rejected().effect(),
        &TestEffect::MinimumMovementSpeed { percent: 25 },
    );

    assert_eq!(
        rejections[1].winner().effect(),
        &TestEffect::MinimumMovementSpeed { percent: 30 },
    );

    assert_synthetic_origin(rejections[1].rejected().origin(), "medium");

    assert_synthetic_origin(rejections[1].winner().origin(), "strong");
}

#[test]
fn detailed_calculation_can_be_consumed_into_output_and_plan() {
    let source =
        SingleEffectSource::new("maximum_life", TestEffect::AddedMaximumLife { amount: 25 });

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let active = collect_active(&collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let calculation = calculator
        .calculate_detailed(&active, TestEffectAccumulator::with_base_maximum_life(100))
        .expect("detailed calculation should succeed");

    let (stats, execution_plan) = calculation.into_parts();

    assert_eq!(stats.maximum_life, 125);

    assert_eq!(execution_plan.len(), 1);

    assert_eq!(execution_plan.selection_rejection_count(), 0,);

    let effects = execution_plan.effects().collect::<Vec<_>>();

    assert_eq!(effects, vec![&TestEffect::AddedMaximumLife { amount: 25 },],);
}

struct DetailedPanicEffectApplier;

impl EffectApplier<TestGame> for DetailedPanicEffectApplier {
    type Accumulator = ();
    type Error = Infallible;

    fn apply_effect(
        &self,
        _effect: &TestEffect,
        _accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        panic!("effect applier must not run after planning error");
    }
}

struct DetailedPanicFinalizer;

impl EffectAccumulatorFinalizer for DetailedPanicFinalizer {
    type Accumulator = ();
    type Output = ();
    type Error = Infallible;

    fn finalize(&self, _accumulator: Self::Accumulator) -> Result<Self::Output, Self::Error> {
        panic!("finalizer must not run after planning or application error");
    }
}

#[test]
fn detailed_calculation_returns_plan_error_before_application() {
    let first = SingleEffectSource::new("first_override", TestEffect::SetMaximumLife { value: 1 });

    let second =
        SingleEffectSource::new("second_override", TestEffect::SetMaximumLife { value: 10 });

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&first);
    collection.collect_from_source(&second);

    let active = collect_active(&collection);

    let calculator = EffectCalculator::new(
        DetailedPanicEffectApplier,
        DetailedPanicFinalizer,
        test_effect_execution_planner(),
    );

    let result = calculator.calculate_detailed(&active, ());

    assert!(matches!(
        result,
        Err(EffectCalculationError::Plan(
            EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                key: TestEffectConflictKey::MaximumLifeOverride,
                ..
            }
        ))
    ));
}

#[test]
fn detailed_calculation_returns_finalize_error() {
    let source = SingleEffectSource::new(
        "overflowing_life",
        TestEffect::AddedMaximumLife { amount: 1 },
    );

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let active = collect_active(&collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let result = calculator.calculate_detailed(
        &active,
        TestEffectAccumulator::with_base_maximum_life(u32::MAX),
    );

    assert!(matches!(
        result,
        Err(EffectCalculationError::Finalize(
            TestEffectFinalizeError::MaximumLifeOverflow
        ))
    ));
}

#[derive(Debug, PartialEq, Eq)]
enum DetailedFactoryError {
    RejectedInput,
}

struct FailingDetailedFactory;

impl EffectAccumulatorFactory for FailingDetailedFactory {
    type Input = TestCalculationInput;
    type Accumulator = ();
    type Error = DetailedFactoryError;

    fn create(&self, _input: &Self::Input) -> Result<Self::Accumulator, Self::Error> {
        Err(DetailedFactoryError::RejectedInput)
    }
}

struct PanicAfterFactoryEffectApplier;

impl EffectApplier<TestGame> for PanicAfterFactoryEffectApplier {
    type Accumulator = ();
    type Error = Infallible;

    fn apply_effect(
        &self,
        _effect: &TestEffect,
        _accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        panic!("effect application must not run after factory error");
    }
}

struct PanicAfterFactoryFinalizer;

impl EffectAccumulatorFinalizer for PanicAfterFactoryFinalizer {
    type Accumulator = ();
    type Output = ();
    type Error = Infallible;

    fn finalize(&self, _accumulator: Self::Accumulator) -> Result<Self::Output, Self::Error> {
        panic!("finalization must not run after factory error");
    }
}

#[test]
fn calculate_from_input_detailed_returns_factory_error_before_calculation() {
    let collection = EffectCollection::<TestGame>::new();

    let active = collect_active(&collection);

    let calculator = EffectCalculator::new(
        PanicAfterFactoryEffectApplier,
        PanicAfterFactoryFinalizer,
        test_effect_execution_planner(),
    );

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let result = calculator.calculate_from_input_detailed(&active, &FailingDetailedFactory, &input);

    assert!(matches!(
        result,
        Err(EffectCalculationFromInputError::CreateAccumulator(
            DetailedFactoryError::RejectedInput
        ))
    ));
}
