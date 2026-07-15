mod support;

use exile_core::effect::EffectApplier;

use support::{
    effect::{TestEffectAccumulator, TestEffectApplier},
    game::TestEffect,
};

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
