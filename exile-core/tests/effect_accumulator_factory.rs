mod support;

use exile_core::effect::EffectAccumulatorFactory;

use support::effect::{TestCalculationInput, TestEffectAccumulatorFactory};

#[test]
fn creates_accumulator_from_calculation_input() {
    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let accumulator = TestEffectAccumulatorFactory
        .create(&input)
        .expect("accumulator creation should succeed");

    assert_eq!(accumulator.base_maximum_life, 100);
}

#[test]
fn creates_accumulator_without_applied_effects() {
    let input = TestCalculationInput {
        base_maximum_life: 100,
    };

    let accumulator = TestEffectAccumulatorFactory
        .create(&input)
        .expect("accumulator creation should succeed");

    assert_eq!(accumulator.added_maximum_life, 0);
    assert_eq!(accumulator.maximum_life_override, None);

    assert!(!accumulator.chaos_immune);

    assert_eq!(accumulator.increased_damage_percent, 0,);

    assert_eq!(accumulator.increased_movement_speed_percent, 0,);
}
