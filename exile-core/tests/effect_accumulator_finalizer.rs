mod support;

use exile_core::effect::EffectAccumulatorFinalizer;

use support::effect::{
    TestEffectAccumulator, TestEffectAccumulatorFinalizer, TestEffectFinalizeError,
};

#[test]
fn finalizes_base_and_added_maximum_life() {
    let accumulator = TestEffectAccumulator {
        base_maximum_life: 100,
        added_maximum_life: 25,
        ..TestEffectAccumulator::default()
    };

    let stats = TestEffectAccumulatorFinalizer
        .finalize(accumulator)
        .expect("finalization should succeed");

    assert_eq!(stats.maximum_life, 125);
}

#[test]
fn maximum_life_override_replaces_calculated_value() {
    let accumulator = TestEffectAccumulator {
        base_maximum_life: 100,
        added_maximum_life: 25,
        maximum_life_override: Some(1),
        ..TestEffectAccumulator::default()
    };

    let stats = TestEffectAccumulatorFinalizer
        .finalize(accumulator)
        .expect("finalization should succeed");

    assert_eq!(stats.maximum_life, 1);
}

#[test]
fn preserves_accumulated_non_life_stats() {
    let accumulator = TestEffectAccumulator {
        chaos_immune: true,
        increased_damage_percent: 20,
        increased_movement_speed_percent: 15,
        ..TestEffectAccumulator::default()
    };

    let stats = TestEffectAccumulatorFinalizer
        .finalize(accumulator)
        .expect("finalization should succeed");

    assert!(stats.chaos_immune);
    assert_eq!(stats.increased_damage_percent, 20);
    assert_eq!(stats.increased_movement_speed_percent, 15,);
}

#[test]
fn returns_error_when_maximum_life_overflows() {
    let accumulator = TestEffectAccumulator {
        base_maximum_life: u32::MAX,
        added_maximum_life: 1,
        ..TestEffectAccumulator::default()
    };

    let result = TestEffectAccumulatorFinalizer.finalize(accumulator);

    assert!(matches!(
        result,
        Err(TestEffectFinalizeError::MaximumLifeOverflow)
    ));
}
