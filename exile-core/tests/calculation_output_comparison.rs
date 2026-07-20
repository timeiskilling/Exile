use exile_core::effect::{
    CalculationComparison, CalculationOutputComparator, NumericStatDifference, StatValueDifference,
};

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();

    assert!(difference < 0.000_001, "expected {expected}, got {actual}",);
}

#[test]
fn numeric_difference_reports_positive_change() {
    let difference = NumericStatDifference::between(100.0, 125.0);

    assert_close(difference.baseline(), 100.0);
    assert_close(difference.candidate(), 125.0);
    assert_close(difference.absolute(), 25.0);

    assert_close(
        difference
            .relative_percent()
            .expect("relative difference should exist"),
        25.0,
    );

    assert!(difference.is_changed());
    assert!(difference.is_positive());
    assert!(!difference.is_negative());
}

#[test]
fn numeric_difference_reports_negative_change() {
    let difference = NumericStatDifference::between(100.0, 80.0);

    assert_close(difference.absolute(), -20.0);

    assert_close(
        difference
            .relative_percent()
            .expect("relative difference should exist"),
        -20.0,
    );

    assert!(difference.is_changed());
    assert!(!difference.is_positive());
    assert!(difference.is_negative());
}

#[test]
fn numeric_difference_has_no_relative_percent_when_baseline_is_zero() {
    let difference = NumericStatDifference::between(0.0, 50.0);

    assert_close(difference.absolute(), 50.0);

    assert_eq!(difference.relative_percent(), None,);

    assert!(difference.is_changed());
    assert!(difference.is_positive());
}

#[test]
fn numeric_difference_reports_unchanged_value() {
    let difference = NumericStatDifference::between(100.0, 100.0);

    assert_close(difference.absolute(), 0.0);

    assert_close(
        difference
            .relative_percent()
            .expect("relative difference should exist"),
        0.0,
    );

    assert!(!difference.is_changed());
    assert!(!difference.is_positive());
    assert!(!difference.is_negative());
}

#[derive(Debug)]
struct TestCalculationOutput {
    maximum_life: u32,
    damage_per_second: f64,
}

#[derive(Debug, PartialEq)]
struct TestCalculationDifference {
    maximum_life: NumericStatDifference,
    damage_per_second: NumericStatDifference,
}

struct TestCalculationOutputComparator;

impl CalculationOutputComparator<TestCalculationOutput> for TestCalculationOutputComparator {
    type Difference = TestCalculationDifference;

    fn compare(
        &self,
        baseline: &TestCalculationOutput,
        candidate: &TestCalculationOutput,
    ) -> Self::Difference {
        TestCalculationDifference {
            maximum_life: NumericStatDifference::between(
                f64::from(baseline.maximum_life),
                f64::from(candidate.maximum_life),
            ),

            damage_per_second: NumericStatDifference::between(
                baseline.damage_per_second,
                candidate.damage_per_second,
            ),
        }
    }
}

#[test]
fn comparator_creates_structured_difference_for_output() {
    let baseline = TestCalculationOutput {
        maximum_life: 1_000,
        damage_per_second: 20_000.0,
    };

    let candidate = TestCalculationOutput {
        maximum_life: 1_150,
        damage_per_second: 21_000.0,
    };

    let comparator = TestCalculationOutputComparator;

    let difference = comparator.compare(&baseline, &candidate);

    assert_close(difference.maximum_life.absolute(), 150.0);

    assert_close(
        difference
            .maximum_life
            .relative_percent()
            .expect("relative difference should exist"),
        15.0,
    );

    assert_close(difference.damage_per_second.absolute(), 1_000.0);

    assert_close(
        difference
            .damage_per_second
            .relative_percent()
            .expect("relative difference should exist"),
        5.0,
    );
}

#[test]
fn value_difference_reports_changed_value() {
    let difference = StatValueDifference::between(false, true);

    assert_eq!(difference.baseline(), &false,);

    assert_eq!(difference.candidate(), &true,);

    assert!(difference.is_changed());
}

#[test]
fn value_difference_reports_unchanged_value() {
    let difference = StatValueDifference::between(Some(80_u32), Some(80_u32));

    assert_eq!(difference.baseline(), &Some(80),);

    assert_eq!(difference.candidate(), &Some(80),);

    assert!(!difference.is_changed());
}

#[test]
fn calculation_comparison_contains_outputs_and_difference() {
    let baseline = TestCalculationOutput {
        maximum_life: 1_000,
        damage_per_second: 20_000.0,
    };

    let candidate = TestCalculationOutput {
        maximum_life: 1_150,
        damage_per_second: 21_000.0,
    };

    let comparison =
        CalculationComparison::between(baseline, candidate, &TestCalculationOutputComparator);

    assert_eq!(comparison.baseline().maximum_life, 1_000,);

    assert_eq!(comparison.candidate().maximum_life, 1_150,);

    assert_close(comparison.difference().maximum_life.absolute(), 150.0);

    assert_close(
        comparison
            .difference()
            .maximum_life
            .relative_percent()
            .expect("maximum life percentage should exist"),
        15.0,
    );

    assert_close(
        comparison.difference().damage_per_second.absolute(),
        1_000.0,
    );

    assert_close(
        comparison
            .difference()
            .damage_per_second
            .relative_percent()
            .expect("DPS percentage should exist"),
        5.0,
    );
}

#[test]
fn calculation_comparison_can_be_consumed_into_parts() {
    let baseline = TestCalculationOutput {
        maximum_life: 100,
        damage_per_second: 1_000.0,
    };

    let candidate = TestCalculationOutput {
        maximum_life: 125,
        damage_per_second: 1_100.0,
    };

    let comparison =
        CalculationComparison::between(baseline, candidate, &TestCalculationOutputComparator);

    let (baseline, candidate, difference) = comparison.into_parts();

    assert_eq!(baseline.maximum_life, 100);
    assert_eq!(candidate.maximum_life, 125);

    assert_close(difference.maximum_life.absolute(), 25.0);

    assert_close(difference.damage_per_second.absolute(), 100.0);
}
