mod support;

use exile_core::effect::{
    EffectApplier, EffectCollection, EffectCollectionApplier, EffectCollectionEvaluator,
    EffectEntry, EffectExecutionPlanner, EffectPlanner, EffectSource,
};

use support::{
    effect::{
        TestEffectAccumulator, TestEffectApplier, TestEffectConditionEvaluator, TestEffectContext,
        TestPassiveNode,
    },
    game::{TestEffect, TestGame},
};

use crate::support::{TestEffectPlanningPolicy, TestEffectSourceId};

struct OrderedEffectSource;

impl EffectSource<TestGame> for OrderedEffectSource {
    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![
            EffectEntry::unconditional(TestEffect::AddedMaximumLife { amount: 25 }),
            EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 1 }),
            EffectEntry::unconditional(TestEffect::IncreasedDamage { percent: 20 }),
        ]
    }

    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic("ordered_effect_source")
    }
}

#[test]
fn applies_all_active_effects_in_collection_order() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&OrderedEffectSource);

    let condition_evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = condition_evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let mut accumulator = TestEffectAccumulator {
        base_maximum_life: 100,
        ..TestEffectAccumulator::default()
    };

    let collection_applier = EffectCollectionApplier::new(TestEffectApplier);

    let planner = EffectExecutionPlanner::new(TestEffectPlanningPolicy);
    let plan = planner
        .plan(&active)
        .expect("effect planning should succeed");

    collection_applier
        .apply_all(&plan, &mut accumulator)
        .expect("all effects should be applied");

    assert_eq!(accumulator.added_maximum_life, 25);
    assert_eq!(accumulator.maximum_life_override, Some(1));
    assert_eq!(accumulator.increased_damage_percent, 20);
}

#[test]
fn does_not_apply_inactive_effects() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    let condition_evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 99,
        enemy_maximum_life: 100,
    };

    let active = condition_evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    assert!(active.is_empty());

    let mut accumulator = TestEffectAccumulator::default();
    let collection_applier = EffectCollectionApplier::new(TestEffectApplier);

    let planner = EffectExecutionPlanner::new(TestEffectPlanningPolicy);
    let plan = planner
        .plan(&active)
        .expect("effect planning should succeed");

    collection_applier
        .apply_all(&plan, &mut accumulator)
        .expect("all effects should be applied");

    assert_eq!(accumulator.increased_damage_percent, 0);
}

#[derive(Debug, Default, PartialEq, Eq)]
struct RecordingAccumulator {
    applied_effects: Vec<&'static str>,
}

#[derive(Debug, PartialEq, Eq)]
enum FailingApplyError {
    SetMaximumLifeRejected,
}

struct FailingEffectApplier;

impl EffectApplier<TestGame> for FailingEffectApplier {
    type Accumulator = RecordingAccumulator;
    type Error = FailingApplyError;

    fn apply_effect(
        &self,
        effect: &TestEffect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        match effect {
            TestEffect::ChaosImmune => {
                accumulator.applied_effects.push("chaos_immune");

                Ok(())
            }

            TestEffect::SetMaximumLife { .. } => Err(FailingApplyError::SetMaximumLifeRejected),

            _ => {
                accumulator.applied_effects.push("other");

                Ok(())
            }
        }
    }
}

#[test]
fn stops_on_first_error_and_keeps_previous_changes() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::ChaosInoculation);

    let condition_evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = condition_evaluator
        .collect_active(&collection, &context)
        .expect("condition evaluation should succeed");

    let mut accumulator = RecordingAccumulator::default();

    let collection_applier = EffectCollectionApplier::new(FailingEffectApplier);

    let planner = EffectExecutionPlanner::new(TestEffectPlanningPolicy);
    let plan = planner
        .plan(&active)
        .expect("effect planning should succeed");

    let result = collection_applier.apply_all(&plan, &mut accumulator);

    assert!(matches!(
        result,
        Err(FailingApplyError::SetMaximumLifeRejected)
    ));

    assert_eq!(accumulator.applied_effects, vec!["chaos_immune"],);
}
