use std::{convert::Infallible, sync::Arc};

use exile_core::{
    effect::{
        BuildCalculationCore, BuildCalculationRunner, BuildCandidateFactory, BuildEffectCollector,
        CalculationComparisonRunner, CalculationOutputComparator, EffectAccumulatorFactory,
        EffectAccumulatorFinalizer, EffectApplier, EffectCollection, EffectCollectionEvaluator,
        EffectConditionEvaluator, EffectExecutionPlanner, EffectPlanningPolicy,
        ItemEffectCollectionError, ItemEffectCollector, ModifierEffectResolver,
        NumericStatDifference,
    },
    game::{Game, ModifierDefinitionIdentity},
    item::{
        ItemEditor, ItemInstance, ItemRule, ItemValidator, ModifierDefinitionProvider,
        ModifierInstanceId, ModifierValidator, Unvalidated, Validated,
    },
};

pub struct MyGame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyItemBase {
    Boots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyItemState {
    pub item_level: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MyModifierDefinitionId {
    MovementSpeed,
}

#[derive(Debug)]
pub struct MyModifierDefinition {
    id: MyModifierDefinitionId,
    required_item_level: u16,
    minimum_roll: u16,
    maximum_roll: u16,
}

impl ModifierDefinitionIdentity for MyModifierDefinition {
    type Id = MyModifierDefinitionId;

    fn modifier_definition_id(&self) -> Self::Id {
        self.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyModifierInstance {
    pub value: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyEffect {
    IncreasedMovementSpeed { percent: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyEffectCondition {
    EnemyOnFullLife,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MyEffectSourceId {
    PassiveNode(u32),
}

impl Game for MyGame {
    type ItemBase = MyItemBase;
    type ItemState = MyItemState;

    type ModifierDefinitionId = MyModifierDefinitionId;
    type ModifierDefinition = MyModifierDefinition;
    type ModifierInstance = MyModifierInstance;

    type Effect = MyEffect;
    type EffectCondition = MyEffectCondition;
    type EffectSourceId = MyEffectSourceId;
}

pub struct MyDefinitionProvider {
    movement_speed: MyModifierDefinition,
}

impl MyDefinitionProvider {
    pub fn new() -> Self {
        Self {
            movement_speed: MyModifierDefinition {
                id: MyModifierDefinitionId::MovementSpeed,
                required_item_level: 1,
                minimum_roll: 1,
                maximum_roll: 40,
            },
        }
    }

    pub fn movement_speed_definition(&self) -> &MyModifierDefinition {
        &self.movement_speed
    }
}

impl Default for MyDefinitionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ModifierDefinitionProvider<MyGame> for MyDefinitionProvider {
    type Error = Infallible;

    fn definition(
        &self,
        id: &MyModifierDefinitionId,
    ) -> Result<&MyModifierDefinition, Self::Error> {
        match id {
            MyModifierDefinitionId::MovementSpeed => Ok(&self.movement_speed),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MyModifierValidationError {
    ItemLevelTooLow {
        required: u16,
        actual: u16,
    },
    RollOutsideAllowedRange {
        minimum: u16,
        maximum: u16,
        actual: u16,
    },
}

pub struct MyModifierValidator;

impl ModifierValidator<MyGame> for MyModifierValidator {
    type Error = MyModifierValidationError;

    fn validate_modifier(
        &self,
        item: &ItemInstance<MyGame, Unvalidated>,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        let item_level = item.state().item_level;

        if item_level < definition.required_item_level {
            return Err(MyModifierValidationError::ItemLevelTooLow {
                required: definition.required_item_level,
                actual: item_level,
            });
        }

        if modifier.value < definition.minimum_roll || modifier.value > definition.maximum_roll {
            return Err(MyModifierValidationError::RollOutsideAllowedRange {
                minimum: definition.minimum_roll,
                maximum: definition.maximum_roll,
                actual: modifier.value,
            });
        }

        Ok(())
    }
}

pub struct MyItemRule;

impl ItemRule<MyGame> for MyItemRule {
    type Error = MyModifierValidationError;

    fn validate_add_modifier(
        &self,
        item: &ItemInstance<MyGame, Unvalidated>,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        MyModifierValidator.validate_modifier(item, definition, modifier)
    }

    fn validate_replace_modifier(
        &self,
        item: &ItemInstance<MyGame, Unvalidated>,
        _target_id: ModifierInstanceId,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        MyModifierValidator.validate_modifier(item, definition, modifier)
    }

    fn validate_replace_state(
        &self,
        _item: &ItemInstance<MyGame, Unvalidated>,
        _new_state: &MyItemState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn validate_remove_modifier(
        &self,
        _item: &ItemInstance<MyGame, Unvalidated>,
        _id: ModifierInstanceId,
        _modifier: &MyModifierInstance,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct MyItemValidator<'a> {
    definitions: &'a MyDefinitionProvider,
    modifier_validator: MyModifierValidator,
}

impl<'a> MyItemValidator<'a> {
    pub fn new(definitions: &'a MyDefinitionProvider) -> Self {
        Self {
            definitions,
            modifier_validator: MyModifierValidator,
        }
    }
}

impl ItemValidator<MyGame> for MyItemValidator<'_> {
    type Error = MyModifierValidationError;

    fn validate_item(&self, item: &ItemInstance<MyGame, Unvalidated>) -> Result<(), Self::Error> {
        for stored_modifier in item.modifiers() {
            let definition = self
                .definitions
                .definition(stored_modifier.definition_id())
                .expect("MyDefinitionProvider uses an infallible lookup");

            self.modifier_validator.validate_modifier(
                item,
                definition,
                stored_modifier.modifier(),
            )?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum MyResolveError {
    DefinitionAndInstanceDoNotMatch,
}

pub struct MyModifierEffectResolver;

impl ModifierEffectResolver<MyGame> for MyModifierEffectResolver {
    type Error = MyResolveError;

    fn resolve_modifier_effects(
        &self,
        definition: &MyModifierDefinition,
        modifier: &MyModifierInstance,
    ) -> Result<Vec<exile_core::effect::EffectEntry<MyGame>>, Self::Error> {
        match definition.id {
            MyModifierDefinitionId::MovementSpeed => {
                Ok(vec![exile_core::effect::EffectEntry::unconditional(
                    MyEffect::IncreasedMovementSpeed {
                        percent: modifier.value,
                    },
                )])
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyConditionContext {
    pub enemy_on_full_life: bool,
}

pub struct MyConditionEvaluator;

impl EffectConditionEvaluator<MyGame> for MyConditionEvaluator {
    type Context = MyConditionContext;
    type Error = Infallible;

    fn evaluate_condition(
        &self,
        condition: &MyEffectCondition,
        context: &Self::Context,
    ) -> Result<bool, Self::Error> {
        match condition {
            MyEffectCondition::EnemyOnFullLife => Ok(context.enemy_on_full_life),
        }
    }
}

pub struct MyPlanningPolicy;

impl EffectPlanningPolicy<MyGame> for MyPlanningPolicy {
    type Phase = u8;
    type Priority = u8;
    type ConflictKey = ();
    type SelectionKey = ();

    fn phase(&self, _effect: &MyEffect) -> Self::Phase {
        0
    }

    fn priority(&self, _effect: &MyEffect) -> Self::Priority {
        0
    }

    fn conflict_key(&self, _effect: &MyEffect) -> Option<Self::ConflictKey> {
        None
    }

    fn selection_key(&self, _effect: &MyEffect) -> Option<Self::SelectionKey> {
        None
    }

    fn prefers(&self, _candidate: &MyEffect, _current: &MyEffect) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyCalculationInput {
    pub base_movement_speed_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyAccumulator {
    movement_speed_percent: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MyFinalStats {
    pub movement_speed_percent: u32,
}

pub struct MyAccumulatorFactory;

impl EffectAccumulatorFactory for MyAccumulatorFactory {
    type Input = MyCalculationInput;
    type Accumulator = MyAccumulator;
    type Error = Infallible;

    fn create(&self, input: &Self::Input) -> Result<Self::Accumulator, Self::Error> {
        Ok(MyAccumulator {
            movement_speed_percent: input.base_movement_speed_percent,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyApplyError {
    MovementSpeedOverflow,
}

pub struct MyEffectApplier;

impl EffectApplier<MyGame> for MyEffectApplier {
    type Accumulator = MyAccumulator;
    type Error = MyApplyError;

    fn apply_effect(
        &self,
        effect: &MyEffect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        match effect {
            MyEffect::IncreasedMovementSpeed { percent } => {
                let updated = accumulator
                    .movement_speed_percent
                    .checked_add(u32::from(*percent))
                    .ok_or(MyApplyError::MovementSpeedOverflow)?;

                accumulator.movement_speed_percent = updated;

                Ok(())
            }
        }
    }
}

pub struct MyFinalizer;

impl EffectAccumulatorFinalizer for MyFinalizer {
    type Accumulator = MyAccumulator;
    type Output = MyFinalStats;
    type Error = Infallible;

    fn finalize(&self, accumulator: Self::Accumulator) -> Result<Self::Output, Self::Error> {
        Ok(MyFinalStats {
            movement_speed_percent: accumulator.movement_speed_percent,
        })
    }
}

#[derive(Clone)]
pub struct MyBuild {
    pub boots: Arc<ItemInstance<MyGame, Validated>>,
}

pub struct MyBuildEffectCollector<'a> {
    item_collector: ItemEffectCollector<'a, MyDefinitionProvider, MyModifierEffectResolver>,
}

impl<'a> MyBuildEffectCollector<'a> {
    pub fn new(
        definition_provider: &'a MyDefinitionProvider,
        resolver: &'a MyModifierEffectResolver,
    ) -> Self {
        Self {
            item_collector: ItemEffectCollector::new(definition_provider, resolver),
        }
    }
}

impl BuildEffectCollector<MyGame> for MyBuildEffectCollector<'_> {
    type Build = MyBuild;
    type Error = ItemEffectCollectionError<Infallible, MyResolveError>;

    fn collect_effects(
        &self,
        build: &Self::Build,
    ) -> Result<EffectCollection<MyGame>, Self::Error> {
        let mut effects = EffectCollection::<MyGame>::new();

        effects.collect_from_item(&self.item_collector, build.boots.as_ref())?;

        Ok(effects)
    }
}

pub struct ReplaceBootsCandidate {
    pub boots: Arc<ItemInstance<MyGame, Validated>>,
}

pub struct MyBuildCandidateFactory;

impl BuildCandidateFactory<MyBuild> for MyBuildCandidateFactory {
    type Candidate = ReplaceBootsCandidate;
    type Error = Infallible;

    fn create_candidate(
        &self,
        current: &MyBuild,
        candidate: &Self::Candidate,
    ) -> Result<MyBuild, Self::Error> {
        let mut candidate_build = current.clone();

        candidate_build.boots = Arc::clone(&candidate.boots);

        Ok(candidate_build)
    }
}

pub struct MyOutputComparator;

impl CalculationOutputComparator<MyFinalStats> for MyOutputComparator {
    type Difference = NumericStatDifference;

    fn compare(&self, baseline: &MyFinalStats, candidate: &MyFinalStats) -> Self::Difference {
        NumericStatDifference::between(
            f64::from(baseline.movement_speed_percent),
            f64::from(candidate.movement_speed_percent),
        )
    }
}

type MyPlanner = EffectExecutionPlanner<MyPlanningPolicy>;

type MyRunner<'a> = BuildCalculationRunner<
    MyBuildEffectCollector<'a>,
    MyConditionEvaluator,
    MyEffectApplier,
    MyFinalizer,
    MyPlanner,
>;

type MyCore<'a> = BuildCalculationCore<
    MyGame,
    MyBuildEffectCollector<'a>,
    MyConditionEvaluator,
    MyEffectApplier,
    MyFinalizer,
    MyPlanner,
    MyAccumulatorFactory,
    MyOutputComparator,
>;

fn create_validated_boots(
    definition_provider: &MyDefinitionProvider,
    movement_speed_percent: u16,
) -> ItemInstance<MyGame, Validated> {
    let movement_speed_definition = definition_provider.movement_speed_definition();

    let mut item = ItemInstance::<MyGame>::new(MyItemBase::Boots, MyItemState { item_level: 86 });

    let editor = ItemEditor::new(MyItemRule);

    editor
        .add_modifier(
            &mut item,
            movement_speed_definition,
            MyModifierInstance {
                value: movement_speed_percent,
            },
        )
        .expect("the movement-speed modifier must be valid");

    let item_validator = MyItemValidator::new(definition_provider);

    item.validate(&item_validator)
        .expect("the boots must pass complete item validation")
}

#[test]
fn test_main() {
    let definition_provider = MyDefinitionProvider::new();

    let modifier_resolver = MyModifierEffectResolver;

    let current_boots = Arc::new(create_validated_boots(&definition_provider, 25));

    let candidate_boots = Arc::new(create_validated_boots(&definition_provider, 35));

    let current_build = MyBuild {
        boots: Arc::clone(&current_boots),
    };

    let build_collector = MyBuildEffectCollector::new(&definition_provider, &modifier_resolver);

    let evaluator = EffectCollectionEvaluator::new(MyConditionEvaluator);

    let planner = EffectExecutionPlanner::new(MyPlanningPolicy);

    let calculator =
        exile_core::effect::EffectCalculator::new(MyEffectApplier, MyFinalizer, planner);

    let runner: MyRunner<'_> = BuildCalculationRunner::new(build_collector, evaluator, calculator);

    let comparison_runner = CalculationComparisonRunner::new(MyOutputComparator);

    let context = MyConditionContext {
        enemy_on_full_life: false,
    };

    let calculation_input = MyCalculationInput {
        base_movement_speed_percent: 100,
    };

    let mut core: MyCore<'_> = BuildCalculationCore::new(
        current_build,
        context,
        calculation_input,
        MyAccumulatorFactory,
        runner,
        comparison_runner,
    );

    let current_movement_speed = core
        .calculate_current()
        .expect("the current build must calculate successfully")
        .movement_speed_percent;

    assert_eq!(current_movement_speed, 125,);

    let candidate = ReplaceBootsCandidate {
        boots: candidate_boots,
    };

    let comparison = core
        .compare_candidate_with(&MyBuildCandidateFactory, &candidate)
        .expect("the candidate build must calculate successfully");

    assert_eq!(comparison.baseline().movement_speed_percent, 125,);

    assert_eq!(comparison.candidate().movement_speed_percent, 135,);

    assert_eq!(comparison.difference().absolute(), 10.0,);
}
