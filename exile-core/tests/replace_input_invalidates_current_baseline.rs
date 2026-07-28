mod support;

use exile_core::effect::{
    BuildCalculationCore, BuildCalculationRunner, CalculationComparisonRunner,
    CalculationOutputComparator, EffectCalculator, EffectCollectionEvaluator,
};

use support::{
    build::{CountingBuildEffectCollector, TestBuild, TestBuildEffectCollector},
    effect::{
        TestCalculationInput, TestEffectAccumulatorFactory, TestEffectAccumulatorFinalizer,
        TestEffectApplier, TestEffectConditionEvaluator, TestEffectContext, TestFinalStats,
        TestModifierEffectResolver, test_effect_execution_planner,
    },
    game::TestGame,
    item::TestModifierDefinitionProvider,
};

struct MaximumLifeComparator;

impl CalculationOutputComparator<TestFinalStats> for MaximumLifeComparator {
    type Difference = i64;

    fn compare(&self, baseline: &TestFinalStats, candidate: &TestFinalStats) -> Self::Difference {
        i64::from(candidate.maximum_life) - i64::from(baseline.maximum_life)
    }
}

#[test]
fn replace_input_invalidates_current_baseline() {
    let definitions = TestModifierDefinitionProvider::new(Vec::new());

    let resolver = TestModifierEffectResolver::default();

    let inner_build_collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let (build_collector, collection_calls) =
        CountingBuildEffectCollector::new(inner_build_collector);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let comparison_runner = CalculationComparisonRunner::new(MaximumLifeComparator);

    let build = TestBuild::new(Vec::new(), Vec::new());

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let initial_input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        build,
        context,
        initial_input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    assert_eq!(collection_calls.get(), 0);
    assert!(core.current_output().is_none());

    let initial_output = core
        .calculate_current()
        .expect("initial calculation should succeed");

    assert_eq!(initial_output.maximum_life, 100,);

    assert_eq!(collection_calls.get(), 1);
    assert!(core.current_output().is_some());

    let replacement_input = TestCalculationInput {
        base_maximum_life: 250,
    };

    let previous_input = core
        .replace_input(replacement_input)
        .expect("input replacement should succeed");

    assert_eq!(
        previous_input,
        TestCalculationInput {
            base_maximum_life: 100,
        },
    );

    assert_eq!(
        core.input(),
        &TestCalculationInput {
            base_maximum_life: 250,
        },
    );

    assert_eq!(collection_calls.get(), 1);
    assert!(core.current_output().is_none());

    let recalculated_output = core
        .calculate_current()
        .expect("calculation with replacement input should succeed");

    assert_eq!(recalculated_output.maximum_life, 250,);

    assert_eq!(collection_calls.get(), 2);

    let stored_output = core
        .current_output()
        .expect("recalculation should create a new baseline");

    assert_eq!(stored_output.maximum_life, 250,);
}
