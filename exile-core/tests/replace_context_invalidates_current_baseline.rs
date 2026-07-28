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
        TestModifierEffectResolver, TestPassiveNode, test_effect_execution_planner,
    },
    game::TestGame,
    item::TestModifierDefinitionProvider,
};

struct IncreasedDamageComparator;

impl CalculationOutputComparator<TestFinalStats> for IncreasedDamageComparator {
    type Difference = i64;

    fn compare(&self, baseline: &TestFinalStats, candidate: &TestFinalStats) -> Self::Difference {
        i64::from(candidate.increased_damage_percent) - i64::from(baseline.increased_damage_percent)
    }
}

#[test]
fn replace_context_invalidates_current_baseline() {
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

    let comparison_runner = CalculationComparisonRunner::new(IncreasedDamageComparator);

    let build = TestBuild::new(Vec::new(), vec![TestPassiveNode::FullLifeDamage]);

    let full_life_context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        build,
        full_life_context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    assert_eq!(collection_calls.get(), 0,);

    assert!(core.current_output().is_none());

    let full_life_output = core
        .calculate_current()
        .expect("full-life calculation should succeed");

    assert_eq!(full_life_output.increased_damage_percent, 20,);

    assert_eq!(collection_calls.get(), 1,);

    assert!(core.current_output().is_some(),);

    let not_full_life_context = TestEffectContext {
        enemy_current_life: 99,
        enemy_maximum_life: 100,
    };

    core.replace_context(not_full_life_context)
        .expect("context replacement should succeed");

    assert!(core.current_output().is_none(),);

    assert_eq!(collection_calls.get(), 1,);

    let not_full_life_output = core
        .calculate_current()
        .expect("not-full-life calculation should succeed");

    assert_eq!(not_full_life_output.increased_damage_percent, 0,);

    assert_eq!(collection_calls.get(), 2,);

    let stored_output = core
        .current_output()
        .expect("new calculation should create a baseline");

    assert_eq!(stored_output.increased_damage_percent, 0,);
}
