mod support;

use std::{cell::Cell, rc::Rc};

use exile_core::{
    effect::{
        BuildCalculationCore, BuildCalculationRunner, BuildEffectCollector,
        CalculationComparisonRunner, CalculationOutputComparator, EffectCalculator,
        EffectCollection, EffectCollectionEvaluator, ItemEffectCollectionError,
        ItemEffectCollector,
    },
    item::{ItemInstance, Validated},
};

use support::{
    effect::{
        TestCalculationInput, TestEffectAccumulatorFactory, TestEffectAccumulatorFinalizer,
        TestEffectApplier, TestEffectConditionEvaluator, TestEffectContext, TestEffectResolveError,
        TestModifierEffectResolver, TestPassiveNode, test_effect_execution_planner,
    },
    game::{
        TestEffect, TestGame, TestItemBase, TestItemState, TestModifier, TestModifierDefinition,
        TestModifierKind,
    },
    item::{
        TestItemValidator, TestModifierDefinitionProvider, TestModifierDefinitionProviderError,
        movement_speed_definition,
    },
};

use crate::support::{TestFinalStats, TestFinalStatsComparator};

struct TestBuild {
    items: Vec<ItemInstance<TestGame, Validated>>,
    passive_nodes: Vec<TestPassiveNode>,
}

impl TestBuild {
    fn new(
        items: Vec<ItemInstance<TestGame, Validated>>,
        passive_nodes: Vec<TestPassiveNode>,
    ) -> Self {
        Self {
            items,
            passive_nodes,
        }
    }

    fn items(&self) -> &[ItemInstance<TestGame, Validated>] {
        &self.items
    }

    fn passive_nodes(&self) -> &[TestPassiveNode] {
        &self.passive_nodes
    }
}

type TestBuildEffectCollectionError =
    ItemEffectCollectionError<TestModifierDefinitionProviderError, TestEffectResolveError>;

struct TestBuildEffectCollector<'a> {
    item_collector:
        ItemEffectCollector<'a, TestModifierDefinitionProvider, TestModifierEffectResolver>,
}

impl<'a> TestBuildEffectCollector<'a> {
    fn new(
        definitions: &'a TestModifierDefinitionProvider,
        resolver: &'a TestModifierEffectResolver,
    ) -> Self {
        Self {
            item_collector: ItemEffectCollector::new(definitions, resolver),
        }
    }
}

impl BuildEffectCollector<TestGame> for TestBuildEffectCollector<'_> {
    type Build = TestBuild;
    type Error = TestBuildEffectCollectionError;

    fn collect_effects(
        &self,
        build: &Self::Build,
    ) -> Result<EffectCollection<TestGame>, Self::Error> {
        let mut effects = EffectCollection::<TestGame>::new();

        effects.collect_from_items(&self.item_collector, build.items().iter())?;

        effects.collect_from_sources(build.passive_nodes().iter());

        Ok(effects)
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

fn build_with_three_movement_speed_items(
    definitions: &TestModifierDefinitionProvider,
) -> TestBuild {
    TestBuild::new(
        vec![
            validated_rolled_item(definitions, TestModifierKind::MovementSpeed, 20),
            validated_rolled_item(definitions, TestModifierKind::MovementSpeed, 25),
            validated_rolled_item(definitions, TestModifierKind::MovementSpeed, 30),
        ],
        vec![TestPassiveNode::FullLifeDamage],
    )
}

fn assert_movement_speed_effect(effect: &TestEffect, expected_percent: u16) {
    match effect {
        TestEffect::IncreasedMovementSpeed { percent } => {
            assert_eq!(*percent, expected_percent);
        }

        other => {
            panic!("expected IncreasedMovementSpeed, found {other:?}");
        }
    }
}

#[test]
fn collects_effects_from_complete_build() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let build = build_with_three_movement_speed_items(&definitions);

    let effects = collector
        .collect_effects(&build)
        .expect("build effect collection should succeed");

    assert_eq!(effects.len(), 4);

    let mut entries = effects.iter();

    assert_movement_speed_effect(
        entries
            .next()
            .expect("first item effect should exist")
            .effect(),
        20,
    );

    assert_movement_speed_effect(
        entries
            .next()
            .expect("second item effect should exist")
            .effect(),
        25,
    );

    assert_movement_speed_effect(
        entries
            .next()
            .expect("third item effect should exist")
            .effect(),
        30,
    );

    assert!(matches!(
        entries
            .next()
            .expect("passive node effect should exist")
            .effect(),
        TestEffect::IncreasedDamage { percent: 20 }
    ));

    assert!(entries.next().is_none());
}

#[test]
fn calculates_final_stats_from_complete_build() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let build = build_with_three_movement_speed_items(&definitions);

    let effects = collector
        .collect_effects(&build)
        .expect("build effect collection should succeed");

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active_effects = evaluator
        .collect_active(&effects, &context)
        .expect("effect evaluation should succeed");

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
        .expect("build calculation should succeed");

    assert_eq!(stats.maximum_life, 100,);

    assert_eq!(stats.increased_movement_speed_percent, 75,);

    assert_eq!(stats.increased_damage_percent, 20,);

    assert!(!stats.chaos_immune);
}

#[test]
fn returns_error_when_one_build_item_cannot_resolve() {
    let unsupported_definition = TestModifierDefinition {
        kind: TestModifierKind::Unsupported,
        required_item_level: 1,
        min_roll: 1,
        max_roll: 10,
    };

    let definitions = TestModifierDefinitionProvider::new(vec![
        movement_speed_definition(),
        unsupported_definition,
    ]);

    let resolver = TestModifierEffectResolver::default();

    let collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let valid_item = validated_rolled_item(&definitions, TestModifierKind::MovementSpeed, 20);

    let unsupported_item = validated_rolled_item(&definitions, TestModifierKind::Unsupported, 5);

    let build = TestBuild::new(vec![valid_item, unsupported_item], Vec::new());

    let result = collector.collect_effects(&build);

    assert!(matches!(
        result,
        Err(ItemEffectCollectionError::Resolver(
            TestEffectResolveError::UnsupportedModifier
        ))
    ));
}

#[test]
fn supports_build_without_items() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let build = TestBuild::new(Vec::new(), vec![TestPassiveNode::ChaosInoculation]);

    let effects = collector
        .collect_effects(&build)
        .expect("source-only build should be collected");

    assert_eq!(effects.len(), 2);

    let mut entries = effects.iter();

    assert!(matches!(
        entries
            .next()
            .expect("chaos immunity effect should exist")
            .effect(),
        TestEffect::ChaosImmune
    ));

    assert!(matches!(
        entries
            .next()
            .expect("maximum life override should exist")
            .effect(),
        TestEffect::SetMaximumLife { value: 1 }
    ));

    assert!(entries.next().is_none());
}

#[test]
fn build_calculation_runner_calculates_complete_build() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let build_collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let build = build_with_three_movement_speed_items(&definitions);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let stats = runner
        .calculate_build(&build, &context, &TestEffectAccumulatorFactory, &input)
        .expect("complete build calculation should succeed");

    assert_eq!(stats.maximum_life, 100,);

    assert_eq!(stats.increased_movement_speed_percent, 75,);

    assert_eq!(stats.increased_damage_percent, 20,);

    assert!(!stats.chaos_immune);
}

fn build_with_one_movement_speed_item(definitions: &TestModifierDefinitionProvider) -> TestBuild {
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
fn build_calculation_core_calculates_current_build_and_stores_output() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let build_collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let build = build_with_three_movement_speed_items(&definitions);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let comparator = TestFinalStatsComparator;
    let comparison_runner = CalculationComparisonRunner::new(comparator);
    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        build,
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    assert!(core.current_output().is_none());

    {
        let output = core
            .calculate_current()
            .expect("current build calculation should succeed");

        assert_eq!(output.maximum_life, 100,);

        assert_eq!(output.increased_movement_speed_percent, 75,);

        assert_eq!(output.increased_damage_percent, 20,);

        assert!(!output.chaos_immune);
    }

    let stored_output = core
        .current_output()
        .expect("calculated output should be stored");

    assert_eq!(stored_output.increased_movement_speed_percent, 75,);

    assert_eq!(stored_output.increased_damage_percent, 20,);
}

#[test]
fn build_calculation_core_invalidates_output_after_build_replacement() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let build_collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let initial_build = build_with_three_movement_speed_items(&definitions);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let comparator = TestFinalStatsComparator;
    let comparison_runner = CalculationComparisonRunner::new(comparator);

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        initial_build,
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    {
        let initial_output = core
            .calculate_current()
            .expect("initial build calculation should succeed");

        assert_eq!(initial_output.increased_movement_speed_percent, 75,);

        assert_eq!(initial_output.increased_damage_percent, 20,);
    }

    assert!(core.current_output().is_some());

    let replacement_build = build_with_one_movement_speed_item(&definitions);

    core.replace_build(replacement_build)
        .expect("build replacement should succeed");

    assert!(core.current_output().is_none());

    {
        let updated_output = core
            .calculate_current()
            .expect("replacement build calculation should succeed");

        assert_eq!(updated_output.increased_movement_speed_percent, 20,);

        assert_eq!(updated_output.increased_damage_percent, 0,);

        assert_eq!(updated_output.maximum_life, 100,);
    }

    let stored_output = core
        .current_output()
        .expect("replacement output should be stored");

    assert_eq!(stored_output.increased_movement_speed_percent, 20,);
}

struct MovementSpeedComparator;

impl CalculationOutputComparator<TestFinalStats> for MovementSpeedComparator {
    type Difference = i64;

    fn compare(&self, baseline: &TestFinalStats, candidate: &TestFinalStats) -> Self::Difference {
        i64::from(candidate.increased_movement_speed_percent)
            - i64::from(baseline.increased_movement_speed_percent)
    }
}

struct CountingBuildEffectCollector<'a> {
    inner: TestBuildEffectCollector<'a>,
    calls: Rc<Cell<usize>>,
}

impl<'a> CountingBuildEffectCollector<'a> {
    fn new(inner: TestBuildEffectCollector<'a>, calls: Rc<Cell<usize>>) -> Self {
        Self { inner, calls }
    }
}

impl BuildEffectCollector<TestGame> for CountingBuildEffectCollector<'_> {
    type Build = TestBuild;
    type Error = TestBuildEffectCollectionError;

    fn collect_effects(
        &self,
        build: &Self::Build,
    ) -> Result<EffectCollection<TestGame>, Self::Error> {
        self.calls.set(self.calls.get() + 1);

        self.inner.collect_effects(build)
    }
}

#[test]
fn compare_candidate_build_creates_missing_baseline() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let build_collector = TestBuildEffectCollector::new(&definitions, &resolver);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let comparison_runner = CalculationComparisonRunner::new(MovementSpeedComparator);

    let current_build = build_with_three_movement_speed_items(&definitions);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        current_build,
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    let candidate_build = TestBuild::new(
        vec![validated_rolled_item(
            &definitions,
            TestModifierKind::MovementSpeed,
            20,
        )],
        Vec::new(),
    );

    assert!(core.current_output().is_none());

    let comparison = core
        .compare_candidate_build(&candidate_build)
        .expect("candidate comparison should succeed");

    assert_eq!(comparison.baseline().increased_movement_speed_percent, 75,);

    assert_eq!(comparison.candidate().increased_movement_speed_percent, 20,);

    assert_eq!(*comparison.difference(), -55,);

    let current_output = core
        .current_output()
        .expect("baseline should be created automatically");

    assert_eq!(current_output.increased_movement_speed_percent, 75,);

    assert_eq!(current_output.increased_damage_percent, 20,);
}

#[test]
fn compare_candidate_build_reuses_baseline_and_does_not_replace_it() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let collection_calls = Rc::new(Cell::new(0));

    let build_collector = CountingBuildEffectCollector::new(
        TestBuildEffectCollector::new(&definitions, &resolver),
        Rc::clone(&collection_calls),
    );

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let comparison_runner = CalculationComparisonRunner::new(MovementSpeedComparator);

    let current_build = build_with_three_movement_speed_items(&definitions);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        current_build,
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    let first_candidate = TestBuild::new(
        vec![validated_rolled_item(
            &definitions,
            TestModifierKind::MovementSpeed,
            20,
        )],
        Vec::new(),
    );

    let second_candidate = TestBuild::new(
        vec![validated_rolled_item(
            &definitions,
            TestModifierKind::MovementSpeed,
            30,
        )],
        Vec::new(),
    );

    let first_comparison = core
        .compare_candidate_build(&first_candidate)
        .expect("first candidate comparison should succeed");

    assert_eq!(collection_calls.get(), 2,);

    assert_eq!(*first_comparison.difference(), -55,);

    let second_comparison = core
        .compare_candidate_build(&second_candidate)
        .expect("second candidate comparison should succeed");

    assert_eq!(collection_calls.get(), 3,);

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
        30,
    );

    assert_eq!(*second_comparison.difference(), -45,);

    let current_output = core
        .current_output()
        .expect("current baseline should still exist");

    assert_eq!(current_output.increased_movement_speed_percent, 75,);
}

#[test]
fn core_reuses_baseline_and_recreates_it_after_build_replacement() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let resolver = TestModifierEffectResolver::default();

    let collection_calls = Rc::new(Cell::new(0));

    let build_collector = CountingBuildEffectCollector::new(
        TestBuildEffectCollector::new(&definitions, &resolver),
        Rc::clone(&collection_calls),
    );

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let runner = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let comparison_runner = CalculationComparisonRunner::new(MovementSpeedComparator);

    let initial_build = build_with_three_movement_speed_items(&definitions);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        initial_build,
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    let first_candidate = build_with_one_movement_speed_item(&definitions);

    assert!(core.current_output().is_none());

    let first_comparison = core
        .compare_candidate_build(&first_candidate)
        .expect("first candidate comparison should succeed");

    assert_eq!(
        first_comparison.baseline().increased_movement_speed_percent,
        75,
    );

    assert_eq!(
        first_comparison
            .candidate()
            .increased_movement_speed_percent,
        20,
    );

    assert_eq!(*first_comparison.difference(), -55,);

    assert_eq!(collection_calls.get(), 2,);

    assert_eq!(
        core.current_output()
            .expect("initial baseline should exist",)
            .increased_movement_speed_percent,
        75,
    );

    let repeated_comparison = core
        .compare_candidate_build(&first_candidate)
        .expect("repeated candidate comparison should succeed");

    assert_eq!(
        repeated_comparison
            .baseline()
            .increased_movement_speed_percent,
        75,
    );

    assert_eq!(*repeated_comparison.difference(), -55,);

    assert_eq!(collection_calls.get(), 3,);

    let replacement_build = build_with_one_movement_speed_item(&definitions);

    core.replace_build(replacement_build)
        .expect("build replacement should succeed");

    assert!(core.current_output().is_none());

    let second_candidate = TestBuild::new(
        vec![validated_rolled_item(
            &definitions,
            TestModifierKind::MovementSpeed,
            30,
        )],
        Vec::new(),
    );

    let comparison_after_replacement = core
        .compare_candidate_build(&second_candidate)
        .expect("comparison after build replacement should succeed");

    assert_eq!(
        comparison_after_replacement
            .baseline()
            .increased_movement_speed_percent,
        20,
    );

    assert_eq!(
        comparison_after_replacement
            .candidate()
            .increased_movement_speed_percent,
        30,
    );

    assert_eq!(*comparison_after_replacement.difference(), 10,);

    assert_eq!(collection_calls.get(), 5,);

    let updated_baseline = core
        .current_output()
        .expect("updated baseline should exist");

    assert_eq!(updated_baseline.increased_movement_speed_percent, 20,);
}
