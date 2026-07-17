mod support;

use exile_core::effect::EffectApplier;

use support::{
    effect::{TestEffectAccumulator, TestEffectApplier},
    game::TestEffect,
};

use crate::support::TestEffectApplyError;

#[test]
fn applies_boolean_effect() {
    let mut accumulator = TestEffectAccumulator::default();

    TestEffectApplier
        .apply_effect(&TestEffect::ChaosImmune, &mut accumulator)
        .expect("effect application should succeed");

    assert!(accumulator.chaos_immune);
}

#[test]
fn applies_set_effect() {
    let mut accumulator = TestEffectAccumulator {
        base_maximum_life: 100,
        ..TestEffectAccumulator::default()
    };

    TestEffectApplier
        .apply_effect(&TestEffect::SetMaximumLife { value: 1 }, &mut accumulator)
        .expect("effect application should succeed");

    assert_eq!(accumulator.maximum_life_override, Some(1),);
}

#[test]
fn applies_added_maximum_life_effect() {
    let mut accumulator = TestEffectAccumulator {
        base_maximum_life: 100,
        ..TestEffectAccumulator::default()
    };

    TestEffectApplier
        .apply_effect(
            &TestEffect::AddedMaximumLife { amount: 25 },
            &mut accumulator,
        )
        .expect("effect application should succeed");

    assert_eq!(accumulator.base_maximum_life, 100);
    assert_eq!(accumulator.added_maximum_life, 25);
    assert_eq!(accumulator.maximum_life_override, None);
}

#[test]
fn physical_damage_maximum_overflow_does_not_partially_update_range() {
    let mut accumulator = TestEffectAccumulator {
        added_physical_damage_min: 10,
        added_physical_damage_max: u32::MAX,
        ..TestEffectAccumulator::default()
    };

    let result = TestEffectApplier.apply_effect(
        &TestEffect::AddedPhysicalDamage { min: 5, max: 1 },
        &mut accumulator,
    );

    assert_eq!(
        result,
        Err(TestEffectApplyError::AddedPhysicalDamageMaximum),
    );

    assert_eq!(accumulator.added_physical_damage_min, 10,);

    assert_eq!(accumulator.added_physical_damage_max, u32::MAX,);
}

#[test]
fn increased_damage_overflow_does_not_mutate_accumulator() {
    let mut accumulator = TestEffectAccumulator {
        increased_damage_percent: u32::MAX,
        ..TestEffectAccumulator::default()
    };

    let result = TestEffectApplier.apply_effect(
        &TestEffect::IncreasedDamage { percent: 1 },
        &mut accumulator,
    );

    assert_eq!(result, Err(TestEffectApplyError::IncreasedDamage,),);

    assert_eq!(accumulator.increased_damage_percent, u32::MAX,);
}
