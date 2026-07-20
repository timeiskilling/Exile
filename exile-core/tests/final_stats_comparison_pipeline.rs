mod support;

use exile_core::effect::{
    CalculationOutputComparator, EffectCalculator, EffectCollection, EffectCollectionEvaluator,
    EffectEntry, EffectSource,
};

use support::{
    effect::{
        TestEffectAccumulator, TestEffectAccumulatorFinalizer, TestEffectApplier,
        TestEffectConditionEvaluator, TestEffectContext, TestFinalStats, TestFinalStatsComparator,
        test_effect_execution_planner,
    },
    game::{TestEffect, TestEffectSourceId, TestGame},
};

struct StaticEffectSource {
    id: &'static str,
    effects: Vec<TestEffect>,
}

impl StaticEffectSource {
    fn new(id: &'static str, effects: Vec<TestEffect>) -> Self {
        Self { id, effects }
    }
}

impl EffectSource<TestGame> for StaticEffectSource {
    fn effect_source_id(&self) -> TestEffectSourceId {
        TestEffectSourceId::Synthetic(self.id)
    }

    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        self.effects
            .iter()
            .cloned()
            .map(EffectEntry::unconditional)
            .collect()
    }
}

fn build_collection(id: &'static str, effects: Vec<TestEffect>) -> EffectCollection<TestGame> {
    let source = StaticEffectSource::new(id, effects);

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&source);

    collection
}

fn calculate_stats(collection: &EffectCollection<TestGame>) -> TestFinalStats {
    let evaluator = EffectCollectionEvaluator::new(TestEffectConditionEvaluator);

    let context = TestEffectContext {
        enemy_current_life: 100,
        enemy_maximum_life: 100,
    };

    let active = evaluator
        .collect_active(collection, &context)
        .expect("effect condition evaluation should succeed");

    let calculator = EffectCalculator::new(
        TestEffectApplier,
        TestEffectAccumulatorFinalizer,
        test_effect_execution_planner(),
    );

    let accumulator = TestEffectAccumulator {
        base_maximum_life: 100,
        ..Default::default()
    };

    calculator
        .calculate(&active, accumulator)
        .expect("effect calculation should succeed")
}

fn assert_close(actual: f64, expected: f64) {
    let difference = (actual - expected).abs();

    assert!(difference < 0.000_001, "expected {expected}, got {actual}",);
}

#[test]
fn compares_baseline_and_candidate_calculations() {
    let baseline_collection = build_collection("baseline", Vec::new());

    let candidate_collection = build_collection(
        "candidate",
        vec![TestEffect::AddedMaximumLife { amount: 25 }],
    );

    let baseline = calculate_stats(&baseline_collection);

    let candidate = calculate_stats(&candidate_collection);

    let difference = TestFinalStatsComparator.compare(&baseline, &candidate);

    assert_eq!(baseline.maximum_life, 100);
    assert_eq!(candidate.maximum_life, 125);

    assert_close(difference.maximum_life.absolute(), 25.0);

    assert_close(
        difference
            .maximum_life
            .relative_percent()
            .expect("maximum life relative difference should exist"),
        25.0,
    );

    assert!(difference.maximum_life.is_positive());

    assert!(difference.maximum_life.is_changed());
}

#[test]
fn reports_negative_difference_when_candidate_is_worse() {
    let baseline_collection = build_collection(
        "equipped_item",
        vec![TestEffect::AddedMaximumLife { amount: 50 }],
    );

    let candidate_collection = build_collection(
        "candidate_item",
        vec![TestEffect::AddedMaximumLife { amount: 20 }],
    );

    let baseline = calculate_stats(&baseline_collection);

    let candidate = calculate_stats(&candidate_collection);

    let difference = TestFinalStatsComparator.compare(&baseline, &candidate);

    assert_eq!(baseline.maximum_life, 150);
    assert_eq!(candidate.maximum_life, 120);

    assert_close(difference.maximum_life.absolute(), -30.0);

    assert_close(
        difference
            .maximum_life
            .relative_percent()
            .expect("maximum life relative difference should exist"),
        -20.0,
    );

    assert!(difference.maximum_life.is_negative());

    assert!(difference.maximum_life.is_changed());
}

#[test]
fn reports_no_final_life_change_when_override_suppresses_added_life() {
    let baseline_collection =
        build_collection("baseline", vec![TestEffect::SetMaximumLife { value: 1 }]);

    let candidate_collection = build_collection(
        "candidate",
        vec![
            TestEffect::AddedMaximumLife { amount: 25 },
            TestEffect::SetMaximumLife { value: 1 },
        ],
    );

    let baseline = calculate_stats(&baseline_collection);

    let candidate = calculate_stats(&candidate_collection);

    let difference = TestFinalStatsComparator.compare(&baseline, &candidate);

    assert_eq!(baseline.maximum_life, 1);
    assert_eq!(candidate.maximum_life, 1);

    assert_close(difference.maximum_life.absolute(), 0.0);

    assert_close(
        difference
            .maximum_life
            .relative_percent()
            .expect("maximum life relative difference should exist"),
        0.0,
    );

    assert!(!difference.maximum_life.is_changed());

    assert!(!difference.maximum_life.is_positive());

    assert!(!difference.maximum_life.is_negative());
}
