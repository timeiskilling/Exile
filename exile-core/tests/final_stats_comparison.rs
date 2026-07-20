mod support;

use exile_core::effect::CalculationOutputComparator;

use support::effect::{TestFinalStats, TestFinalStatsComparator};

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();
    assert!(difference < 0.000_001, "expected {expected}, got {actual}",);
}

#[test]
fn compares_real_final_stats() {
    let baseline = TestFinalStats {
        maximum_life: 100,
        chaos_immune: false,
        increased_damage_percent: 20,
        increased_movement_speed_percent: 10,
        added_physical_damage_min: 5,
        added_physical_damage_max: 10,
        minimum_movement_speed_percent: 0,
        maximum_movement_speed_percent: None,
    };

    let candidate = TestFinalStats {
        maximum_life: 125,
        chaos_immune: true,
        increased_damage_percent: 30,
        increased_movement_speed_percent: 5,
        added_physical_damage_min: 7,
        added_physical_damage_max: 15,
        minimum_movement_speed_percent: 20,
        maximum_movement_speed_percent: Some(80),
    };

    let comparator = TestFinalStatsComparator;

    let difference = comparator.compare(&baseline, &candidate);

    assert_close(difference.maximum_life.absolute(), 25.0);

    assert_close(
        difference
            .maximum_life
            .relative_percent()
            .expect("maximum life relative difference should exist"),
        25.0,
    );

    assert!(difference.chaos_immune.is_changed());

    assert_eq!(difference.chaos_immune.baseline(), &false,);

    assert_eq!(difference.chaos_immune.candidate(), &true,);

    assert_close(difference.increased_damage_percent.absolute(), 10.0);

    assert_close(
        difference
            .increased_damage_percent
            .relative_percent()
            .expect("damage relative difference should exist"),
        50.0,
    );

    assert_close(difference.increased_movement_speed_percent.absolute(), -5.0);

    assert_close(
        difference
            .increased_movement_speed_percent
            .relative_percent()
            .expect("movement speed relative difference should exist"),
        -50.0,
    );

    assert_close(difference.added_physical_damage_min.absolute(), 2.0);

    assert_close(difference.added_physical_damage_max.absolute(), 5.0);

    assert_close(difference.minimum_movement_speed_percent.absolute(), 20.0);

    assert_eq!(
        difference.minimum_movement_speed_percent.relative_percent(),
        None,
    );

    assert!(difference.maximum_movement_speed_percent.is_changed());

    assert_eq!(difference.maximum_movement_speed_percent.baseline(), &None,);

    assert_eq!(
        difference.maximum_movement_speed_percent.candidate(),
        &Some(80),
    );
}

#[test]
fn identical_final_stats_have_no_changes() {
    let baseline = TestFinalStats {
        maximum_life: 100,
        chaos_immune: false,

        increased_damage_percent: 20,

        increased_movement_speed_percent: 10,

        added_physical_damage_min: 5,
        added_physical_damage_max: 10,

        minimum_movement_speed_percent: 0,

        maximum_movement_speed_percent: Some(80),
    };

    let candidate = TestFinalStats {
        maximum_life: 100,
        chaos_immune: false,

        increased_damage_percent: 20,

        increased_movement_speed_percent: 10,

        added_physical_damage_min: 5,
        added_physical_damage_max: 10,

        minimum_movement_speed_percent: 0,

        maximum_movement_speed_percent: Some(80),
    };

    let difference = TestFinalStatsComparator.compare(&baseline, &candidate);

    assert!(!difference.maximum_life.is_changed());

    assert!(!difference.chaos_immune.is_changed());

    assert!(!difference.increased_damage_percent.is_changed());

    assert!(!difference.increased_movement_speed_percent.is_changed());

    assert!(!difference.added_physical_damage_min.is_changed());

    assert!(!difference.added_physical_damage_max.is_changed());

    assert!(!difference.minimum_movement_speed_percent.is_changed());

    assert!(!difference.maximum_movement_speed_percent.is_changed());
}
