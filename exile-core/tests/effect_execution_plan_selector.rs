mod support;

use exile_core::effect::{
    EffectCollection, EffectCollectionEvaluator, EffectEntry, EffectOrigin, EffectSource,
    calculation::{EffectExecutionPlan, EffectExecutionPlanSelector, EffectPriorityResolver},
};

use support::{
    effect::{
        TestEffectConditionEvaluator, TestEffectContext, TestEffectPhaseResolver,
        TestEffectPriority, TestEffectPriorityResolver,
    },
    game::{TestEffect, TestEffectSourceId, TestGame},
};

use crate::support::TestEffectStrengthResolver;

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

struct UniformPriorityResolver;

impl EffectPriorityResolver<TestGame> for UniformPriorityResolver {
    type Priority = TestEffectPriority;

    fn priority(&self, _effect: &TestEffect) -> Self::Priority {
        TestEffectPriority::Normal
    }
}

fn select_effects<P>(
    collection: &EffectCollection<TestGame>,
    priority_resolver: &P,
) -> Vec<TestEffect>
where
    P: EffectPriorityResolver<TestGame>,
{
    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(collection, &context)
        .expect("condition evaluation should succeed");

    let plan = EffectExecutionPlan::build(&active, &TestEffectPhaseResolver, priority_resolver);

    let selector = EffectExecutionPlanSelector::new(TestEffectStrengthResolver);

    let selected = selector.select(plan);

    selected.effects().cloned().collect()
}

fn select_effects_with_origins<P>(
    collection: &EffectCollection<TestGame>,
    priority_resolver: &P,
) -> Vec<(TestEffect, EffectOrigin<TestGame>)>
where
    P: EffectPriorityResolver<TestGame>,
{
    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(collection, &context)
        .expect("condition evaluation should succeed");

    let plan = EffectExecutionPlan::build(&active, &TestEffectPhaseResolver, priority_resolver);

    let selector = EffectExecutionPlanSelector::new(TestEffectStrengthResolver);

    let selected = selector.select(plan);

    selected
        .iter()
        .map(|entry| (entry.effect().clone(), entry.origin().clone()))
        .collect()
}

#[test]
fn selects_strongest_effect_for_same_key() {
    let source = StaticEffectSource {
        id: "strongest_selection",
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

    let effects = select_effects(&collection, &TestEffectPriorityResolver);

    assert_eq!(
        effects,
        vec![TestEffect::MinimumMovementSpeed { percent: 30 },],
    );
}

#[test]
fn keeps_existing_winner_when_later_effects_are_weaker() {
    let source = StaticEffectSource {
        id: "existing_winner",
        effects: || {
            vec![
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 30 }),
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 20 }),
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 25 }),
            ]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let effects = select_effects(&collection, &TestEffectPriorityResolver);

    assert_eq!(
        effects,
        vec![TestEffect::MinimumMovementSpeed { percent: 30 },],
    );
}

#[test]
fn replaces_previous_winner_when_stronger_effect_is_found() {
    let source = StaticEffectSource {
        id: "replacement_winner",
        effects: || {
            vec![
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 20 }),
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 25 }),
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 30 }),
            ]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let effects = select_effects(&collection, &TestEffectPriorityResolver);

    assert_eq!(
        effects,
        vec![TestEffect::MinimumMovementSpeed { percent: 30 },],
    );
}

#[test]
fn keeps_first_effect_when_strengths_are_equal() {
    let first_source = StaticEffectSource {
        id: "first_equal_strength",
        effects: || {
            vec![EffectEntry::unconditional(
                TestEffect::MinimumMovementSpeed { percent: 30 },
            )]
        },
    };

    let second_source = StaticEffectSource {
        id: "second_equal_strength",
        effects: || {
            vec![EffectEntry::unconditional(
                TestEffect::MinimumMovementSpeed { percent: 30 },
            )]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&first_source);
    collection.collect_from_source(&second_source);

    let entries = select_effects_with_origins(&collection, &TestEffectPriorityResolver);

    assert_eq!(entries.len(), 1);

    assert_eq!(
        entries[0].0,
        TestEffect::MinimumMovementSpeed { percent: 30 },
    );

    assert!(matches!(
        entries[0].1,
        EffectOrigin::Source(TestEffectSourceId::Synthetic("first_equal_strength",),),
    ));
}

#[test]
fn preserves_effects_without_strength_key() {
    let source = StaticEffectSource {
        id: "ordinary_effects",
        effects: || {
            vec![
                EffectEntry::unconditional(TestEffect::AddedMaximumLife { amount: 25 }),
                EffectEntry::unconditional(TestEffect::AddedMaximumLife { amount: 40 }),
                EffectEntry::unconditional(TestEffect::IncreasedDamage { percent: 20 }),
            ]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let effects = select_effects(&collection, &TestEffectPriorityResolver);

    assert_eq!(
        effects,
        vec![
            TestEffect::AddedMaximumLife { amount: 25 },
            TestEffect::AddedMaximumLife { amount: 40 },
            TestEffect::IncreasedDamage { percent: 20 },
        ],
    );
}

#[test]
fn winner_keeps_its_original_plan_position() {
    let source = StaticEffectSource {
        id: "winner_position",
        effects: || {
            vec![
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 20 }),
                EffectEntry::unconditional(TestEffect::ChaosImmune),
                EffectEntry::unconditional(TestEffect::MinimumMovementSpeed { percent: 30 }),
            ]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let effects = select_effects(&collection, &UniformPriorityResolver);

    assert_eq!(
        effects,
        vec![
            TestEffect::ChaosImmune,
            TestEffect::MinimumMovementSpeed { percent: 30 },
        ],
    );
}

#[test]
fn winner_preserves_its_origin() {
    let weak_source = StaticEffectSource {
        id: "weak",
        effects: || {
            vec![EffectEntry::unconditional(
                TestEffect::MinimumMovementSpeed { percent: 20 },
            )]
        },
    };

    let strong_source = StaticEffectSource {
        id: "strong",
        effects: || {
            vec![EffectEntry::unconditional(
                TestEffect::MinimumMovementSpeed { percent: 30 },
            )]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&weak_source);
    collection.collect_from_source(&strong_source);

    let entries = select_effects_with_origins(&collection, &TestEffectPriorityResolver);

    assert_eq!(entries.len(), 1);

    assert_eq!(
        entries[0].0,
        TestEffect::MinimumMovementSpeed { percent: 30 },
    );

    assert!(matches!(
        entries[0].1,
        EffectOrigin::Source(TestEffectSourceId::Synthetic("strong")),
    ));
}

#[test]
fn preserves_single_effect_in_strength_group() {
    let source = StaticEffectSource {
        id: "single_grouped_effect",
        effects: || {
            vec![EffectEntry::unconditional(
                TestEffect::MinimumMovementSpeed { percent: 25 },
            )]
        },
    };

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    let effects = select_effects(&collection, &TestEffectPriorityResolver);

    assert_eq!(
        effects,
        vec![TestEffect::MinimumMovementSpeed { percent: 25 },],
    );
}

#[test]
fn selecting_empty_plan_returns_empty_plan() {
    let collection = EffectCollection::<TestGame>::new();

    let effects = select_effects(&collection, &TestEffectPriorityResolver);

    assert!(effects.is_empty());
}
