mod support;

use exile_core::effect::{
    EffectCollection, EffectCollectionEvaluator, EffectEntry, EffectSource,
    calculation::{
        EffectExecutionPlanValidationError, EffectExecutionPlanner, EffectPlanner,
        EffectStrengthResolver,
    },
};

use support::{
    TestEffectConflictKeyResolver,
    effect::{
        TestEffectConditionEvaluator, TestEffectContext, TestEffectPhaseResolver,
        TestEffectPriorityResolver, test_effect_execution_planner,
    },
    game::{TestEffect, TestEffectSourceId, TestGame},
};

use crate::support::TestEffectConflictKey;

struct StaticEffectSource {
    id: &'static str,
    effects: fn() -> Vec<EffectEntry<TestGame>>,
}

impl EffectSource<TestGame> for StaticEffectSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic(self.id)
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        (self.effects)()
    }
}

fn collect_active<'a>(
    collection: &'a EffectCollection<TestGame>,
) -> exile_core::effect::ActiveEffectCollection<'a, TestGame> {
    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    evaluator
        .collect_active(collection, &context)
        .expect("condition evaluation should succeed")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OverrideStrengthKey {
    MaximumLifeOverride,
}

struct OverrideStrengthResolver;

impl EffectStrengthResolver<TestGame> for OverrideStrengthResolver {
    type Key = OverrideStrengthKey;
    type Strength = u32;

    fn strength(&self, effect: &TestEffect) -> Option<(Self::Key, Self::Strength)> {
        match effect {
            TestEffect::SetMaximumLife { value } => {
                Some((OverrideStrengthKey::MaximumLifeOverride, *value))
            }
            _ => None,
        }
    }
}

#[test]
fn planner_selects_strongest_effect() {
    let source = StaticEffectSource {
        id: "strongest_movement_speed",
        effects: || {
            vec![
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 20 }),
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 30 }),
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 25 }),
            ]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let active = collect_active(&collection);

    let planner = test_effect_execution_planner();

    let plan = planner.plan(&active).expect("planning should succeed");

    let effects = plan.effects().collect::<Vec<_>>();

    assert_eq!(effects.len(), 1);

    assert_eq!(
        effects[0],
        &TestEffect::MinimumMovementSpeed { percent: 30 },
    );
}

#[test]
fn planner_validates_conflicts_before_strength_selection() {
    let source = StaticEffectSource {
        id: "conflicting_overrides",
        effects: || {
            vec![
                EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 1 }),
                EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 10 }),
            ]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let active = collect_active(&collection);

    let planner = EffectExecutionPlanner::new(
        TestEffectPhaseResolver,
        TestEffectConflictKeyResolver,
        TestEffectPriorityResolver,
        OverrideStrengthResolver,
    );

    let result = planner.plan(&active);

    assert!(matches!(
        result,
        Err(
            EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
                key: TestEffectConflictKey::MaximumLifeOverride,
                ..
            }
        )
    ));
}
