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
        TestFinalStats, TestModifierEffectResolver, TestPassiveNode, test_effect_execution_planner,
    },
    game::{TestGame, TestItemBase, TestItemState, TestModifier, TestModifierKind},
    item::{
        TestItemValidator, TestModifierDefinitionProvider, TestModifierDefinitionProviderError,
        movement_speed_definition,
    },
};

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

struct MovementSpeedComparator;

impl CalculationOutputComparator<TestFinalStats> for MovementSpeedComparator {
    type Difference = i64;

    fn compare(&self, baseline: &TestFinalStats, candidate: &TestFinalStats) -> Self::Difference {
        i64::from(candidate.increased_movement_speed_percent)
            - i64::from(baseline.increased_movement_speed_percent)
    }
}

fn validated_movement_speed_item(
    definitions: &TestModifierDefinitionProvider,
    roll: u16,
) -> ItemInstance<TestGame, Validated> {
    let validator = TestItemValidator::new(definitions);

    ItemInstance::<TestGame>::from_parts(
        TestItemBase { is_boots: true },
        TestItemState { item_level: 86 },
        vec![(
            TestModifierKind::MovementSpeed,
            TestModifier::Rolled { roll },
        )],
    )
    .validate(&validator)
    .expect("movement speed item should be valid")
}

fn build_with_three_movement_speed_items(
    definitions: &TestModifierDefinitionProvider,
) -> TestBuild {
    TestBuild::new(
        vec![
            validated_movement_speed_item(definitions, 20),
            validated_movement_speed_item(definitions, 25),
            validated_movement_speed_item(definitions, 30),
        ],
        vec![TestPassiveNode::FullLifeDamage],
    )
}

fn build_with_one_movement_speed_item(
    definitions: &TestModifierDefinitionProvider,
    roll: u16,
) -> TestBuild {
    TestBuild::new(
        vec![validated_movement_speed_item(definitions, roll)],
        Vec::new(),
    )
}

#[test]
fn repeated_current_calculation_produces_the_same_output() {
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

    let build = build_with_three_movement_speed_items(&definitions);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        build,
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    assert!(core.current_output().is_none());

    let first_output = *core
        .calculate_current()
        .expect("first current calculation should succeed");

    let second_output = *core
        .calculate_current()
        .expect("second current calculation should succeed");

    assert_eq!(first_output, second_output,);

    assert_eq!(first_output.maximum_life, 100,);

    assert_eq!(first_output.increased_movement_speed_percent, 75,);

    assert_eq!(first_output.increased_damage_percent, 20,);

    assert_eq!(core.current_output(), Some(&second_output),);
}

#[test]
fn repeated_candidate_comparison_reuses_the_same_baseline() {
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

    let candidate_build = build_with_one_movement_speed_item(&definitions, 20);

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

    let first_comparison = core
        .compare_candidate_build(&candidate_build)
        .expect("first candidate comparison should succeed");

    assert_eq!(collection_calls.get(), 2,);

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

    let second_comparison = core
        .compare_candidate_build(&candidate_build)
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
        20,
    );

    assert_eq!(*second_comparison.difference(), -55,);

    let stored_baseline = core
        .current_output()
        .expect("current baseline should remain stored");

    assert_eq!(stored_baseline.increased_movement_speed_percent, 75,);

    assert_eq!(stored_baseline.increased_damage_percent, 20,);
}

#[test]
fn build_replacement_invalidates_and_recreates_baseline() {
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

    let initial_output = *core
        .calculate_current()
        .expect("initial current calculation should succeed");

    assert_eq!(collection_calls.get(), 1,);

    assert_eq!(initial_output.increased_movement_speed_percent, 75,);

    assert_eq!(initial_output.increased_damage_percent, 20,);

    assert_eq!(core.current_output(), Some(&initial_output),);

    let replacement_build = build_with_one_movement_speed_item(&definitions, 20);

    core.replace_build(replacement_build)
        .expect("build replacement should succeed");

    assert!(core.current_output().is_none());

    let candidate_build = build_with_one_movement_speed_item(&definitions, 30);

    let comparison = core
        .compare_candidate_build(&candidate_build)
        .expect("comparison after build replacement should succeed");

    assert_eq!(collection_calls.get(), 3,);

    assert_eq!(comparison.baseline().increased_movement_speed_percent, 20,);

    assert_eq!(comparison.baseline().increased_damage_percent, 0,);

    assert_eq!(comparison.candidate().increased_movement_speed_percent, 30,);

    assert_eq!(*comparison.difference(), 10,);

    let updated_baseline = core
        .current_output()
        .expect("replacement build baseline should exist");

    assert_eq!(updated_baseline.increased_movement_speed_percent, 20,);

    assert_eq!(updated_baseline.increased_damage_percent, 0,);
}
