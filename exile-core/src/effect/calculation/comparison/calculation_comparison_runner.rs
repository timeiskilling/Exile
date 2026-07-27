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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CandidateComparisonError<R, E> {
    StaleBaseline {
        baseline_revision: R,
        current_revision: R,
    },
    Calculation(E),
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

pub type CalculationBaselineOutput<R, F> = CalculationBaseline<R, FinalizedCalculationOutput<F>>;

pub type CalculationBaselineFromInputResult<R, G, A, F, P, Factory> =
    Result<CalculationBaselineOutput<R, F>, CalculationFromInputErrorFor<G, A, F, P, Factory>>;

pub type CandidateComparisonFromInputError<R, G, A, F, P, Factory> =
    CandidateComparisonError<R, CalculationFromInputErrorFor<G, A, F, P, Factory>>;

pub type CandidateComparisonFromInputResult<R, G, A, F, P, Factory, C> = Result<
    CalculationComparisonOutput<F, C>,
    CandidateComparisonFromInputError<R, G, A, F, P, Factory>,
>;

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

    pub fn calculate_baseline_from_input<R, G, A, F, P, Factory>(
        &self,
        revision: R,
        calculator: &EffectCalculator<A, F, P>,
        baseline_effects: &ActiveEffectCollection<'_, G>,
        factory: &Factory,
        input: &Factory::Input,
    ) -> CalculationBaselineFromInputResult<R, G, A, F, P, Factory>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
    {
        let output = calculator.calculate_from_input(baseline_effects, factory, input)?;

        Ok(CalculationBaseline::new(revision, output))
    }

    pub fn compare_candidate_from_input<R, G, A, F, P, Factory>(
        &self,
        calculator: &EffectCalculator<A, F, P>,
        baseline: &CalculationBaselineOutput<R, F>,
        current_revision: &R,
        candidate_effects: &ActiveEffectCollection<'_, G>,
        factory: &Factory,
        input: &Factory::Input,
    ) -> CandidateComparisonFromInputResult<R, G, A, F, P, Factory, C>
    where
        R: Clone + PartialEq,
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        F::Output: Clone,
        P: EffectPlanner<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
        C: CalculationOutputComparator<F::Output>,
    {
        if baseline.revision() != current_revision {
            return Err(CandidateComparisonError::StaleBaseline {
                baseline_revision: baseline.revision().clone(),
                current_revision: current_revision.clone(),
            });
        }

        let candidate = calculator
            .calculate_from_input(candidate_effects, factory, input)
            .map_err(CandidateComparisonError::Calculation)?;

        Ok(CalculationComparison::between(
            baseline.output().clone(),
            candidate,
            &self.comparator,
        ))
    }

    pub fn compare_outputs<O>(
        &self,
        baseline: O,
        candidate: O,
    ) -> CalculationComparison<O, C::Difference>
    where
        C: CalculationOutputComparator<O>,
    {
        let difference = self.comparator.compare(&baseline, &candidate);

        CalculationComparison::new(baseline, candidate, difference)
    }
}
