
mod support;

use exile_core::{
    effect::{
        BuildCalculationCore, BuildCalculationRunner, CalculationComparisonRunner,
        CalculationOutputComparator, EffectCalculator, EffectCollectionEvaluator,
    },
    item::{ItemInstance, Validated},
};

use support::{
    build::{
        CountingBuildEffectCollector, TestBuild, TestBuildEffectCollector,
    },
    effect::{
        TestCalculationInput, TestEffectAccumulatorFactory, TestEffectAccumulatorFinalizer,
        TestEffectApplier, TestEffectConditionEvaluator, TestEffectContext, TestFinalStats,
        TestModifierEffectResolver, test_effect_execution_planner,
    },
    game::{
        TestGame, TestItemBase, TestItemState, TestModifier, TestModifierKind,
    },
    item::{
        TestItemValidator, TestModifierDefinitionProvider, movement_speed_definition,
    },
};

struct MovementSpeedComparator;

impl CalculationOutputComparator<TestFinalStats> for MovementSpeedComparator {
    type Difference = i64;

    fn compare(
        &self,
        baseline: &TestFinalStats,
        candidate: &TestFinalStats,
    ) -> Self::Difference {
        i64::from(candidate.increased_movement_speed_percent)
            - i64::from(baseline.increased_movement_speed_percent)
    }
}

fn validated_rolled_item(
    definitions: &TestModifierDefinitionProvider,
    kind: TestModifierKind,
    roll: u16,
) -> ItemInstance<TestGame, Validated> {
    let validator = TestItemValidator::new(definitions);

    ItemInstance::<TestGame>::from_parts(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
        vec![(kind, TestModifier::Rolled { roll })],
    )
    .validate(&validator)
    .expect("test item should be valid")
}

fn current_build(
    definitions: &TestModifierDefinitionProvider,
) -> TestBuild {
    TestBuild::new(
        vec![
            validated_rolled_item(
                definitions,
                TestModifierKind::MovementSpeed,
                20,
            ),
            validated_rolled_item(
                definitions,
                TestModifierKind::MovementSpeed,
                25,
            ),
            validated_rolled_item(
                definitions,
                TestModifierKind::MovementSpeed,
                30,
            ),
        ],
        Vec::new(),
    )
}

fn candidate_build(
    definitions: &TestModifierDefinitionProvider,
) -> TestBuild {
    TestBuild::new(
        vec![validated_rolled_item(
            definitions,
            TestModifierKind::MovementSpeed,
            20,
        )],
        Vec::new(),
    )
}

#[test]
fn repeated_candidate_comparisons_reuse_current_baseline() {
    let definitions =
        TestModifierDefinitionProvider::new(vec![
            movement_speed_definition(),
        ]);

    let resolver = TestModifierEffectResolver::default();

    let inner_build_collector =
        TestBuildEffectCollector::new(
            &definitions,
            &resolver,
        );

    let (
        build_collector,
        collection_calls,
    ) = CountingBuildEffectCollector::new(
        inner_build_collector,
    );

    let evaluator =
        EffectCollectionEvaluator::new(
            TestEffectConditionEvaluator,
        );

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(
        build_collector,
        evaluator,
        calculator,
    );

    let comparison_runner =
        CalculationComparisonRunner::new(
            MovementSpeedComparator,
        );

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core =
        BuildCalculationCore::<
            TestGame,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new(
            current_build(&definitions),
            context,
            input,
            TestEffectAccumulatorFactory,
            runner,
            comparison_runner,
        );

    let candidate = candidate_build(&definitions);

    assert_eq!(collection_calls.get(), 0);
    assert!(core.current_output().is_none());

    let first_comparison = core
        .compare_candidate_build(&candidate)
        .expect(
            "first candidate comparison should succeed",
        );

    assert_eq!(collection_calls.get(), 2);

    assert_eq!(
        first_comparison
            .baseline()
            .increased_movement_speed_percent,
        75,
    );

    assert_eq!(
        first_comparison
            .candidate()
            .increased_movement_speed_percent,
        20,
    );

    assert_eq!(
        *first_comparison.difference(),
        -55,
    );

    let stored_baseline = core
        .current_output()
        .expect(
            "first comparison should create a baseline",
        );

    assert_eq!(
        stored_baseline
            .increased_movement_speed_percent,
        75,
    );

    let second_comparison = core
        .compare_candidate_build(&candidate)
        .expect(
            "second candidate comparison should succeed",
        );

    assert_eq!(collection_calls.get(), 3);

    assert_eq!(
        second_comparison
            .baseline()
            .increased_movement_speed_percent,
        75,
    );

    assert_eq!(
        second_comparison
            .candidate()
            .increased_movement_speed_percent,
        20,
    );

    assert_eq!(
        *second_comparison.difference(),
        -55,
    );

    let stored_baseline = core
        .current_output()
        .expect(
            "second comparison should preserve the baseline",
        );

    assert_eq!(
        stored_baseline
            .increased_movement_speed_percent,
        75,
    );

    assert_eq!(
        stored_baseline.maximum_life,
        100,
    );
}
