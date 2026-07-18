mod support;

use exile_core::effect::{
    EffectCollection, EffectCollectionEvaluator, EffectEntry, EffectOrigin, EffectSource,
    calculation::{
        EffectExecutionPlan, EffectExecutionPlanValidationError, EffectExecutionPlanValidator,
    },
};

use support::{
    TestEffect, TestEffectConditionEvaluator, TestEffectConflictKey, TestEffectConflictKeyResolver,
    TestEffectContext, TestEffectPhaseResolver, TestEffectSourceId, TestGame,
};

use crate::support::TestEffectPriorityResolver;

struct MaximumLifeOverrideSource {
    value: u32,
    source_id: &'static str,
}

impl EffectSource<TestGame> for MaximumLifeOverrideSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic(self.source_id)
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![EffectEntry::unconditional(TestEffect::SetMaximumLife {
            value: self.value,
        })]
    }
}

struct AddedMaximumLifeSource {
    amount: u16,
    source_id: &'static str,
}

impl EffectSource<TestGame> for AddedMaximumLifeSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic(self.source_id)
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        vec![EffectEntry::unconditional(TestEffect::AddedMaximumLife {
            amount: self.amount,
        })]
    }
}

fn active_effects(
    collection: &EffectCollection<TestGame>,
) -> exile_core::effect::ActiveEffectCollection<'_, TestGame> {
    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    evaluator
        .collect_active(collection, &context)
        .expect("condition evaluation should succeed")
}

#[test]
fn rejects_two_maximum_life_overrides() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&MaximumLifeOverrideSource {
        value: 1,
        source_id: "first_override",
    });

    collection.collect_from_source(&MaximumLifeOverrideSource {
        value: 10,
        source_id: "second_override",
    });

    let active = active_effects(&collection);

    let plan = EffectExecutionPlan::build(
        &active,
        &TestEffectPhaseResolver,
        &TestEffectPriorityResolver,
    );

    let validator = EffectExecutionPlanValidator::new(TestEffectConflictKeyResolver);

    let error = validator
        .validate(&plan)
        .expect_err("two overrides should conflict");

    assert!(matches!(
        error,
        EffectExecutionPlanValidationError::ConflictingExclusiveEffects {
            key: TestEffectConflictKey::MaximumLifeOverride,
            first_origin: EffectOrigin::Source(TestEffectSourceId::Synthetic("first_override")),
            second_origin: EffectOrigin::Source(TestEffectSourceId::Synthetic("second_override")),
        },
    ));
}

#[test]
fn allows_multiple_additive_effects() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&AddedMaximumLifeSource {
        amount: 25,
        source_id: "first_added_life",
    });

    collection.collect_from_source(&AddedMaximumLifeSource {
        amount: 40,
        source_id: "second_added_life",
    });

    let active = active_effects(&collection);

    let plan = EffectExecutionPlan::build(
        &active,
        &TestEffectPhaseResolver,
        &TestEffectPriorityResolver,
    );

    let validator = EffectExecutionPlanValidator::new(TestEffectConflictKeyResolver);

    let result = validator.validate(&plan);

    assert!(matches!(result, Ok(())));
}

#[test]
fn allows_single_maximum_life_override() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&MaximumLifeOverrideSource {
        value: 1,
        source_id: "single_override",
    });

    let active = active_effects(&collection);

    let plan = EffectExecutionPlan::build(
        &active,
        &TestEffectPhaseResolver,
        &TestEffectPriorityResolver,
    );
    
    let validator = EffectExecutionPlanValidator::new(TestEffectConflictKeyResolver);

    assert!(matches!(validator.validate(&plan), Ok(())));
}
