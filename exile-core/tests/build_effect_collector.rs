mod support;

use exile_core::{
    effect::{
        BuildEffectCollector, EffectCalculator, EffectCollection, EffectCollectionEvaluator,
        ItemEffectCollectionError, ItemEffectCollector,
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
