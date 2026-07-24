mod support;

use std::{cell::Cell, rc::Rc};

use exile_core::{
    effect::{
        BuildCalculationCore, BuildCalculationError, BuildCalculationRunner,
        BuildCandidateComparisonError, BuildEffectCollector, CalculationComparisonRunner,
        CalculationOutputComparator, EffectCalculator, EffectCollection, EffectCollectionEvaluator,
        ItemEffectCollectionError, ItemEffectCollector,
    },
    item::{ItemInstance, Validated},
};

use support::{
    effect::{
        TestCalculationInput, TestEffectAccumulatorFactory, TestEffectAccumulatorFinalizer,
        TestEffectApplier, TestEffectConditionEvaluator, TestEffectContext, TestEffectResolveError,
        TestFinalStats, TestModifierEffectResolver, TestPassiveNode, test_effect_execution_planner,
    },
    game::{
        TestGame, TestItemBase, TestItemState, TestModifier, TestModifierDefinition,
        TestModifierKind,
    },
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

fn unsupported_definition() -> TestModifierDefinition {
    TestModifierDefinition {
        kind: TestModifierKind::Unsupported,
        required_item_level: 1,
        min_roll: 1,
        max_roll: 10,
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

fn valid_build(definitions: &TestModifierDefinitionProvider) -> TestBuild {
    TestBuild::new(
        vec![
            validated_rolled_item(definitions, TestModifierKind::MovementSpeed, 20),
            validated_rolled_item(definitions, TestModifierKind::MovementSpeed, 25),
            validated_rolled_item(definitions, TestModifierKind::MovementSpeed, 30),
        ],
        Vec::new(),
    )
}

fn valid_candidate_build(definitions: &TestModifierDefinitionProvider) -> TestBuild {
    TestBuild::new(
        vec![validated_rolled_item(
            definitions,
            TestModifierKind::MovementSpeed,
            20,
        )],
        Vec::new(),
    )
}

fn invalid_build(definitions: &TestModifierDefinitionProvider) -> TestBuild {
    TestBuild::new(
        vec![validated_rolled_item(
            definitions,
            TestModifierKind::Unsupported,
            5,
        )],
        Vec::new(),
    )
}

#[test]
fn candidate_calculation_error_preserves_current_baseline() {
    let definitions = TestModifierDefinitionProvider::new(vec![
        movement_speed_definition(),
        unsupported_definition(),
    ]);

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

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        valid_build(&definitions),
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    let current_output = core
        .calculate_current()
        .expect("current build calculation should succeed");

    assert_eq!(current_output.increased_movement_speed_percent, 75,);

    let candidate = invalid_build(&definitions);

    let result = core.compare_candidate_build(&candidate);

    assert!(matches!(
        result,
        Err(BuildCandidateComparisonError::Candidate(
            BuildCalculationError::Collect(ItemEffectCollectionError::Resolver(
                TestEffectResolveError::UnsupportedModifier
            ))
        ))
    ));

    let stored_output = core
        .current_output()
        .expect("candidate error must preserve current baseline");

    assert_eq!(stored_output.increased_movement_speed_percent, 75,);

    assert_eq!(stored_output.maximum_life, 100,);
}

#[test]
fn current_calculation_error_does_not_create_baseline() {
    let definitions = TestModifierDefinitionProvider::new(vec![
        movement_speed_definition(),
        unsupported_definition(),
    ]);

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

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        invalid_build(&definitions),
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    let candidate = valid_candidate_build(&definitions);

    assert!(core.current_output().is_none());

    let result = core.compare_candidate_build(&candidate);

    assert!(matches!(
        result,
        Err(BuildCandidateComparisonError::Current(
            BuildCalculationError::Collect(ItemEffectCollectionError::Resolver(
                TestEffectResolveError::UnsupportedModifier
            ))
        ))
    ));

    assert_eq!(collection_calls.get(), 1,);

    assert!(core.current_output().is_none());
}

#[test]
fn failed_calculation_after_build_replacement_does_not_restore_old_baseline() {
    let definitions = TestModifierDefinitionProvider::new(vec![
        movement_speed_definition(),
        unsupported_definition(),
    ]);

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

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let mut core = BuildCalculationCore::<TestGame, _, _, _, _, _, _, _>::new(
        valid_build(&definitions),
        context,
        input,
        TestEffectAccumulatorFactory,
        runner,
        comparison_runner,
    );

    let initial_output = core
        .calculate_current()
        .expect("initial current calculation should succeed");

    assert_eq!(initial_output.increased_movement_speed_percent, 75,);

    assert!(core.current_output().is_some());

    core.replace_build(invalid_build(&definitions))
        .expect("build replacement should succeed");

    assert!(core.current_output().is_none());

    assert!(matches!(
        core.calculate_current(),
        Err(BuildCalculationError::Collect(
            ItemEffectCollectionError::Resolver(TestEffectResolveError::UnsupportedModifier)
        ))
    ));

    assert!(core.current_output().is_none());
}
