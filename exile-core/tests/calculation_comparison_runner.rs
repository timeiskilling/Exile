mod support;

use std::cell::Cell;

use exile_core::effect::{
    ActiveEffectCollection, CalculationComparisonError, CalculationComparisonRunner,
    EffectAccumulatorFactory, EffectCalculationFromInputError, EffectCalculator, EffectCollection,
    EffectCollectionEvaluator, EffectEntry, EffectExecutionPlanValidationError, EffectSource,
};

use support::{
    effect::{
        TestCalculationInput, TestEffectAccumulatorFactory, TestEffectAccumulatorFinalizer,
        TestEffectApplier, TestEffectConditionEvaluator, TestEffectConflictKey, TestEffectContext,
        TestFinalStatsComparator, test_effect_execution_planner,
    },
    game::{TestEffect, TestEffectSourceId, TestGame},
};

use crate::support::TestEffectAccumulator;

struct StaticEffectSource {
    id: &'static str,
    effects: Vec<TestEffect>,
}

impl StaticEffectSource {
    fn new(id: &'static str, effects: Vec<TestEffect>) -> Self {
        Self { id, effects }
    }
}

impl EffectSource<TestGame> for StaticEffectSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic(self.id)
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        self.effects
            .iter()
            .cloned()
            .map(EffectEntry::unconditional)
            .collect()
    }
}

fn build_collection(id: &'static str, effects: Vec<TestEffect>) -> EffectCollection<TestGame> {
    let source = StaticEffectSource::new(id, effects);

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    collection
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
        .expect("effect condition evaluation should succeed")
}

fn _test_calculator() -> EffectCalculator<
    TestEffectApplier,
    TestEffectAccumulatorFinalizer,
    impl exile_core::effect::EffectPlanner<TestGame>,
> {
    EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    )
}

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();

    assert!(difference < 0.000_001, "expected {expected}, got {actual}",);
}

#[test]
fn runner_calculates_and_compares_baseline_and_candidate() {
    let baseline_collection = build_collection("baseline", Vec::new());

    let candidate_collection = build_collection(
        "candidate",
        vec![TestEffect::AddedMaximumLife { amount: 25 }],
    );

    let baseline_active = collect_active(&baseline_collection);

    let candidate_active = collect_active(&candidate_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let comparison = runner
        .compare_from_input(
            &calculator,
            &baseline_active,
            &candidate_active,
            &TestEffectAccumulatorFactory,
            &input,
        )
        .expect("baseline and candidate comparison should succeed");

    assert_eq!(comparison.baseline().maximum_life, 100,);

    assert_eq!(comparison.candidate().maximum_life, 125,);

    assert_close(comparison.difference().maximum_life.absolute(), 25.0);

    assert_close(
        comparison
            .difference()
            .maximum_life
            .relative_percent()
            .expect("maximum life percentage should exist"),
        25.0,
    );

    assert!(comparison.difference().maximum_life.is_positive());
}

#[test]
fn runner_reports_negative_difference_for_worse_candidate() {
    let baseline_collection = build_collection(
        "equipped_item",
        vec![TestEffect::AddedMaximumLife { amount: 50 }],
    );

    let candidate_collection = build_collection(
        "candidate_item",
        vec![TestEffect::AddedMaximumLife { amount: 20 }],
    );

    let baseline_active = collect_active(&baseline_collection);

    let candidate_active = collect_active(&candidate_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let comparison = runner
        .compare_from_input(
            &calculator,
            &baseline_active,
            &candidate_active,
            &TestEffectAccumulatorFactory,
            &input,
        )
        .expect("baseline and candidate comparison should succeed");

    assert_eq!(comparison.baseline().maximum_life, 150,);

    assert_eq!(comparison.candidate().maximum_life, 120,);

    assert_close(comparison.difference().maximum_life.absolute(), -30.0);

    assert_close(
        comparison
            .difference()
            .maximum_life
            .relative_percent()
            .expect("maximum life percentage should exist"),
        -20.0,
    );

    assert!(comparison.difference().maximum_life.is_negative());
}

#[test]
fn runner_marks_baseline_calculation_error() {
    let baseline_collection = build_collection(
        "invalid_baseline",
        vec![
            TestEffect::SetMaximumLife { value: 1 },
            TestEffect::SetMaximumLife { value: 10 },
        ],
    );

    let candidate_collection = build_collection("candidate", Vec::new());

    let baseline_active = collect_active(&baseline_collection);

    let candidate_active = collect_active(&candidate_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let result = runner.compare_from_input(
        &calculator,
        &baseline_active,
        &candidate_active,
        &TestEffectAccumulatorFactory,
        &input,
    );

    assert!(matches!(
        result,
        Err(CalculationComparisonError::Baseline(
            EffectCalculationFromInputError::Plan(
                EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                    key: TestEffectConflictKey::MaximumLifeOverride,
                    ..
                }
            )
        ))
    ));
}

#[test]
fn runner_marks_candidate_calculation_error() {
    let baseline_collection = build_collection("baseline", Vec::new());

    let candidate_collection = build_collection(
        "invalid_candidate",
        vec![
            TestEffect::SetMaximumLife { value: 1 },
            TestEffect::SetMaximumLife { value: 10 },
        ],
    );

    let baseline_active = collect_active(&baseline_collection);

    let candidate_active = collect_active(&candidate_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let result = runner.compare_from_input(
        &calculator,
        &baseline_active,
        &candidate_active,
        &TestEffectAccumulatorFactory,
        &input,
    );

    assert!(matches!(
        result,
        Err(CalculationComparisonError::Candidate(
            EffectCalculationFromInputError::Plan(
                EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                    key: TestEffectConflictKey::MaximumLifeOverride,
                    ..
                }
            )
        ))
    ));
}

#[test]
fn runner_compares_final_results_after_override() {
    let baseline_collection =
        build_collection("baseline", vec![TestEffect::SetMaximumLife { value: 1 }]);

    let candidate_collection = build_collection(
        "candidate",
        vec![
            TestEffect::AddedMaximumLife { amount: 25 },
            TestEffect::SetMaximumLife { value: 1 },
        ],
    );

    let baseline_active = collect_active(&baseline_collection);

    let candidate_active = collect_active(&candidate_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let comparison = runner
        .compare_from_input(
            &calculator,
            &baseline_active,
            &candidate_active,
            &TestEffectAccumulatorFactory,
            &input,
        )
        .expect("baseline and candidate comparison should succeed");

    assert_eq!(comparison.baseline().maximum_life, 1,);

    assert_eq!(comparison.candidate().maximum_life, 1,);

    assert_close(comparison.difference().maximum_life.absolute(), 0.0);

    assert!(!comparison.difference().maximum_life.is_changed());
}

struct CountingAccumulatorFactory {
    calls: Cell<usize>,
}

impl CountingAccumulatorFactory {
    fn new() -> Self {
        Self {
            calls: Cell::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.get()
    }
}

impl EffectAccumulatorFactory for CountingAccumulatorFactory {
    type Input = TestCalculationInput;
    type Accumulator = TestEffectAccumulator;

    type Error = <TestEffectAccumulatorFactory as EffectAccumulatorFactory>::Error;

    fn create(&self, input: &Self::Input) -> Result<Self::Accumulator, Self::Error> {
        self.calls.set(self.calls.get() + 1);

        TestEffectAccumulatorFactory.create(input)
    }
}

#[test]
fn runner_reuses_baseline_for_multiple_candidates() {
    let baseline_collection = build_collection("baseline", Vec::new());

    let first_candidate_collection = build_collection(
        "first_candidate",
        vec![TestEffect::AddedMaximumLife { amount: 25 }],
    );

    let second_candidate_collection = build_collection(
        "second_candidate",
        vec![TestEffect::AddedMaximumLife { amount: 50 }],
    );

    let baseline_active = collect_active(&baseline_collection);

    let first_candidate_active = collect_active(&first_candidate_collection);

    let second_candidate_active = collect_active(&second_candidate_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let factory = CountingAccumulatorFactory::new();

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let baseline = runner
        .calculate_baseline_from_input(&calculator, &baseline_active, &factory, &input)
        .expect("baseline calculation should succeed");

    assert_eq!(factory.calls(), 1);

    assert_eq!(baseline.output().maximum_life, 100,);

    let first_comparison = runner
        .compare_candidate_from_input(
            &calculator,
            &baseline,
            &first_candidate_active,
            &factory,
            &input,
        )
        .expect("first candidate comparison should succeed");

    assert_eq!(factory.calls(), 2);

    assert_eq!(first_comparison.baseline().maximum_life, 100,);

    assert_eq!(first_comparison.candidate().maximum_life, 125,);

    assert_close(first_comparison.difference().maximum_life.absolute(), 25.0);

    let second_comparison = runner
        .compare_candidate_from_input(
            &calculator,
            &baseline,
            &second_candidate_active,
            &factory,
            &input,
        )
        .expect("second candidate comparison should succeed");

    assert_eq!(factory.calls(), 3);

    assert_eq!(second_comparison.baseline().maximum_life, 100,);

    assert_eq!(second_comparison.candidate().maximum_life, 150,);

    assert_close(second_comparison.difference().maximum_life.absolute(), 50.0);

    assert_eq!(baseline.output().maximum_life, 100,);
}

#[test]
fn baseline_snapshot_returns_baseline_calculation_error() {
    let baseline_collection = build_collection(
        "invalid_baseline",
        vec![
            TestEffect::SetMaximumLife { value: 1 },
            TestEffect::SetMaximumLife { value: 10 },
        ],
    );

    let baseline_active = collect_active(&baseline_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let result = runner.calculate_baseline_from_input(
        &calculator,
        &baseline_active,
        &TestEffectAccumulatorFactory,
        &input,
    );

    assert!(matches!(
        result,
        Err(EffectCalculationFromInputError::Plan(
            EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                key: TestEffectConflictKey::MaximumLifeOverride,
                ..
            }
        ))
    ));
}

#[test]
fn cached_baseline_comparison_returns_candidate_error() {
    let baseline_collection = build_collection("baseline", Vec::new());

    let candidate_collection = build_collection(
        "invalid_candidate",
        vec![
            TestEffect::SetMaximumLife { value: 1 },
            TestEffect::SetMaximumLife { value: 10 },
        ],
    );

    let baseline_active = collect_active(&baseline_collection);

    let candidate_active = collect_active(&candidate_collection);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = CalculationComparisonRunner::new(TestFinalStatsComparator);

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let baseline = runner
        .calculate_baseline_from_input(
            &calculator,
            &baseline_active,
            &TestEffectAccumulatorFactory,
            &input,
        )
        .expect("baseline calculation should succeed");

    let result = runner.compare_candidate_from_input(
        &calculator,
        &baseline,
        &candidate_active,
        &TestEffectAccumulatorFactory,
        &input,
    );

    assert!(matches!(
        result,
        Err(EffectCalculationFromInputError::Plan(
            EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                key: TestEffectConflictKey::MaximumLifeOverride,
                ..
            }
        ))
    ));
}
