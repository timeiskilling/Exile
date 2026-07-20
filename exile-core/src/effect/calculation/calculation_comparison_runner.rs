use crate::{
    effect::{
        ActiveEffectCollection, CalculationBaseline, CalculationComparison,
        CalculationOutputComparator, EffectAccumulatorFactory, EffectAccumulatorFinalizer,
        EffectApplier, EffectCalculationFromInputError, EffectCalculator, EffectPlanner,
    },
    game::Game,
};

#[derive(Debug)]
pub enum CalculationComparisonError<E> {
    Baseline(E),
    Candidate(E),
}

pub type FinalizedCalculationOutput<F> = <F as EffectAccumulatorFinalizer>::Output;

pub type CalculationOutputDifference<F, C> =
    <C as CalculationOutputComparator<FinalizedCalculationOutput<F>>>::Difference;

pub type CalculationComparisonOutput<F, C> =
    CalculationComparison<FinalizedCalculationOutput<F>, CalculationOutputDifference<F, C>>;

pub type CalculationFromInputErrorFor<G, A, F, P, Factory> = EffectCalculationFromInputError<
    <Factory as EffectAccumulatorFactory>::Error,
    <P as EffectPlanner<G>>::Error,
    <A as EffectApplier<G>>::Error,
    <F as EffectAccumulatorFinalizer>::Error,
>;

pub type CalculationComparisonFromInputError<G, A, F, P, Factory> =
    CalculationComparisonError<CalculationFromInputErrorFor<G, A, F, P, Factory>>;

pub type CalculationComparisonFromInputResult<G, A, F, P, Factory, C> = Result<
    CalculationComparisonOutput<F, C>,
    CalculationComparisonFromInputError<G, A, F, P, Factory>,
>;

pub type CalculationBaselineFromInputResult<G, A, F, P, Factory> = Result<
    CalculationBaseline<<F as EffectAccumulatorFinalizer>::Output>,
    CalculationFromInputErrorFor<G, A, F, P, Factory>,
>;

pub type CandidateComparisonFromInputResult<G, A, F, P, Factory, C> =
    Result<CalculationComparisonOutput<F, C>, CalculationFromInputErrorFor<G, A, F, P, Factory>>;

pub struct CalculationComparisonRunner<C> {
    comparator: C,
}

impl<C> CalculationComparisonRunner<C> {
    pub fn new(comparator: C) -> Self {
        Self { comparator }
    }

    pub fn comparator(&self) -> &C {
        &self.comparator
    }

    pub fn into_comparator(self) -> C {
        self.comparator
    }

    pub fn compare_from_input<G, A, F, P, Factory>(
        &self,
        calculator: &EffectCalculator<A, F, P>,
        baseline_effects: &ActiveEffectCollection<'_, G>,
        candidate_effects: &ActiveEffectCollection<'_, G>,
        factory: &Factory,
        input: &Factory::Input,
    ) -> CalculationComparisonFromInputResult<G, A, F, P, Factory, C>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
        C: CalculationOutputComparator<F::Output>,
    {
        let baseline = calculator
            .calculate_from_input(baseline_effects, factory, input)
            .map_err(CalculationComparisonError::Baseline)?;

        let candidate = calculator
            .calculate_from_input(candidate_effects, factory, input)
            .map_err(CalculationComparisonError::Candidate)?;

        Ok(CalculationComparison::between(
            baseline,
            candidate,
            &self.comparator,
        ))
    }

    pub fn calculate_baseline_from_input<G, A, F, P, Factory>(
        &self,
        calculator: &EffectCalculator<A, F, P>,
        baseline_effects: &ActiveEffectCollection<'_, G>,
        factory: &Factory,
        input: &Factory::Input,
    ) -> CalculationBaselineFromInputResult<G, A, F, P, Factory>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
    {
        let output = calculator.calculate_from_input(baseline_effects, factory, input)?;

        Ok(CalculationBaseline::new(output))
    }

    pub fn compare_candidate_from_input<G, A, F, P, Factory>(
        &self,
        calculator: &EffectCalculator<A, F, P>,
        baseline: &CalculationBaseline<F::Output>,
        candidate_effects: &ActiveEffectCollection<'_, G>,
        factory: &Factory,
        input: &Factory::Input,
    ) -> CandidateComparisonFromInputResult<G, A, F, P, Factory, C>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        F::Output: Clone,
        P: EffectPlanner<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
        C: CalculationOutputComparator<F::Output>,
    {
        let candidate = calculator.calculate_from_input(candidate_effects, factory, input)?;

        Ok(CalculationComparison::between(
            baseline.output().clone(),
            candidate,
            &self.comparator,
        ))
    }
}
