mod support;

use exile_core::effect::{
    EffectCollection, EffectCollectionEvaluator, EffectEntry, EffectOrigin, EffectSource,
    calculation::EffectExecutionPlan,
};

use support::{
    TestEffect, TestEffectConditionEvaluator, TestEffectContext, TestEffectPhaseResolver,
    TestEffectSourceId, TestGame,
};

struct UnorderedPhaseSource;

impl EffectSource<TestGame> for UnorderedPhaseSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic("unordered_phase_source")
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![
            EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 1 }),
            EffectEntry::unconditional(TestEffect::IncreasedDamage { percent: 20 }),
            EffectEntry::unconditional(TestEffect::AddedMaximumLife { amount: 25 }),
            EffectEntry::unconditional(TestEffect::ChaosImmune),
            EffectEntry::unconditional(TestEffect::AddedPhysicalDamage { min: 10, max: 20 }),
            EffectEntry::unconditional(TestEffect::IncreasedMovementSpeed { percent: 15 }),
        ]
    }
}

#[test]
fn orders_active_effects_by_phase() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&UnorderedPhaseSource);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let plan = EffectExecutionPlan::build(&active, &TestEffectPhaseResolver);

    let effects = plan.effects().collect::<Vec<_>>();

    assert_eq!(effects.len(), 6);

    assert_eq!(effects[0], &TestEffect::AddedMaximumLife { amount: 25 },);

    assert_eq!(
        effects[1],
        &TestEffect::AddedPhysicalDamage { min: 10, max: 20 },
    );

    assert_eq!(effects[2], &TestEffect::IncreasedDamage { percent: 20 },);

    assert_eq!(
        effects[3],
        &TestEffect::IncreasedMovementSpeed { percent: 15 },
    );

    assert_eq!(effects[4], &TestEffect::SetMaximumLife { value: 1 },);

    assert_eq!(effects[5], &TestEffect::ChaosImmune,);
}

#[test]
fn execution_plan_preserves_effect_origins() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&UnorderedPhaseSource);

    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let plan = EffectExecutionPlan::build(&active, &TestEffectPhaseResolver);

    for entry in &plan {
        assert!(matches!(
            entry.origin(),
            &EffectOrigin::<TestGame>::Source(TestEffectSourceId::Synthetic(
                "unordered_phase_source",
            ),),
        ));
    }
}
