use std::{marker::PhantomData, mem};

use crate::{
    effect::{
        BuildCalculationErrorFor, BuildCalculationRunner, BuildCandidateFactory,
        BuildEffectCollector, CalculationBaseline, CalculationComparison,
        CalculationComparisonRunner, CalculationOutputComparator, EffectAccumulatorFactory,
        EffectAccumulatorFinalizer, EffectApplier, EffectConditionEvaluator, EffectPlanner,
    },
    game::Game,
};

pub type BuildCalculationCoreOutput<F> = <F as EffectAccumulatorFinalizer>::Output;

pub type BuildCalculationCoreError<G, BC, E, A, F, P, Factory> =
    BuildCalculationErrorFor<G, BC, E, A, F, P, Factory>;

pub type BuildCalculationCoreResult<'a, G, BC, E, A, F, P, Factory> = Result<
    &'a BuildCalculationCoreOutput<F>,
    BuildCalculationCoreError<G, BC, E, A, F, P, Factory>,
>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildCalculationCoreMutationError {
    GenerationOverflow,
}

#[derive(Debug)]
pub enum BuildCandidateComparisonError<E> {
    Current(E),
    Candidate(E),
}

#[derive(Debug)]
pub enum BuildCandidatePreparationError<CreateError, CompareError> {
    Create(CreateError),
    Compare(CompareError),
}

pub type BuildCandidatePreparationErrorFor<G, BC, E, A, F, P, Factory, CF> =
    BuildCandidatePreparationError<
        <CF as BuildCandidateFactory<<BC as BuildEffectCollector<G>>::Build>>::Error,
        BuildCandidateComparisonErrorFor<G, BC, E, A, F, P, Factory>,
    >;

pub type BuildCandidatePreparationResult<G, BC, E, A, F, P, Factory, C, CF> = Result<
    BuildCalculationCoreComparison<F, C>,
    BuildCandidatePreparationErrorFor<G, BC, E, A, F, P, Factory, CF>,
>;

pub type BuildCandidateComparisonErrorFor<G, BC, E, A, F, P, Factory> =
    BuildCandidateComparisonError<BuildCalculationCoreError<G, BC, E, A, F, P, Factory>>;

pub type BuildCalculationCoreDifference<F, C> =
    <C as CalculationOutputComparator<BuildCalculationCoreOutput<F>>>::Difference;

pub type BuildCalculationCoreComparison<F, C> =
    CalculationComparison<BuildCalculationCoreOutput<F>, BuildCalculationCoreDifference<F, C>>;

pub type BuildCandidateComparisonResult<G, BC, E, A, F, P, Factory, C> = Result<
    BuildCalculationCoreComparison<F, C>,
    BuildCandidateComparisonErrorFor<G, BC, E, A, F, P, Factory>,
>;

pub type BuildCalculationCoreOperationResult<G, BC, E, A, F, P, Factory> =
    Result<(), BuildCalculationCoreError<G, BC, E, A, F, P, Factory>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CoreGeneration(u64);

impl CoreGeneration {
    fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

pub struct BuildCalculationCore<G, BC, E, A, F, P, Factory, C>
where
    G: Game,
    BC: BuildEffectCollector<G>,
    E: EffectConditionEvaluator<G>,
    A: EffectApplier<G>,
    F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
    P: EffectPlanner<G>,
    Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
{
    build: BC::Build,
    context: E::Context,
    input: Factory::Input,
    factory: Factory,
    runner: BuildCalculationRunner<BC, E, A, F, P>,
    generation: CoreGeneration,
    baseline: Option<CalculationBaseline<CoreGeneration, F::Output>>,
    marker: PhantomData<fn() -> G>,
    comparison_runner: CalculationComparisonRunner<C>,
}

impl<G, BC, E, A, F, P, Factory, C> BuildCalculationCore<G, BC, E, A, F, P, Factory, C>
where
    G: Game,
    BC: BuildEffectCollector<G>,
    E: EffectConditionEvaluator<G>,
    A: EffectApplier<G>,
    F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
    P: EffectPlanner<G>,
    Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
    C: CalculationOutputComparator<F::Output>,
{
    pub fn new(
        build: BC::Build,
        context: E::Context,
        input: Factory::Input,
        factory: Factory,
        runner: BuildCalculationRunner<BC, E, A, F, P>,
        comparison_runner: CalculationComparisonRunner<C>,
    ) -> Self {
        Self {
            build,
            context,
            input,
            factory,
            runner,
            generation: CoreGeneration(0),
            baseline: None,
            marker: PhantomData,
            comparison_runner,
        }
    }

    fn ensure_baseline(
        &mut self,
    ) -> BuildCalculationCoreOperationResult<G, BC, E, A, F, P, Factory> {
        if self.baseline.is_none() {
            self.calculate_current()?;
        }

        Ok(())
    }

    pub fn compare_candidate_build(
        &mut self,
        candidate_build: &BC::Build,
    ) -> BuildCandidateComparisonResult<G, BC, E, A, F, P, Factory, C>
    where
        F::Output: Clone,
        C: CalculationOutputComparator<F::Output>,
    {
        self.ensure_baseline()
            .map_err(BuildCandidateComparisonError::Current)?;

        let candidate_output = self
            .runner
            .calculate_build(candidate_build, &self.context, &self.factory, &self.input)
            .map_err(BuildCandidateComparisonError::Candidate)?;

        let baseline_output = self
            .baseline
            .as_ref()
            .expect("baseline exists after ensure_baseline")
            .output()
            .clone();

        Ok(self
            .comparison_runner
            .compare_outputs(baseline_output, candidate_output))
    }

    pub fn build(&self) -> &BC::Build {
        &self.build
    }

    pub fn context(&self) -> &E::Context {
        &self.context
    }

    pub fn input(&self) -> &Factory::Input {
        &self.input
    }

    pub fn factory(&self) -> &Factory {
        &self.factory
    }

    pub fn runner(&self) -> &BuildCalculationRunner<BC, E, A, F, P> {
        &self.runner
    }

    pub fn current_output(&self) -> Option<&F::Output> {
        self.baseline.as_ref().map(CalculationBaseline::output)
    }

    pub fn calculate_current(
        &mut self,
    ) -> BuildCalculationCoreResult<'_, G, BC, E, A, F, P, Factory> {
        let output =
            self.runner
                .calculate_build(&self.build, &self.context, &self.factory, &self.input)?;

        let baseline = self
            .baseline
            .insert(CalculationBaseline::new(self.generation, output));

        Ok(baseline.output())
    }

    pub fn replace_build(
        &mut self,
        build: BC::Build,
    ) -> Result<BC::Build, BuildCalculationCoreMutationError> {
        let next_generation = self.next_generation()?;

        let previous = mem::replace(&mut self.build, build);

        self.invalidate_baseline(next_generation);

        Ok(previous)
    }

    pub fn replace_context(
        &mut self,
        context: E::Context,
    ) -> Result<E::Context, BuildCalculationCoreMutationError> {
        let next_generation = self.next_generation()?;

        let previous = mem::replace(&mut self.context, context);

        self.invalidate_baseline(next_generation);

        Ok(previous)
    }

    pub fn replace_input(
        &mut self,
        input: Factory::Input,
    ) -> Result<Factory::Input, BuildCalculationCoreMutationError> {
        let next_generation = self.next_generation()?;

        let previous = mem::replace(&mut self.input, input);

        self.invalidate_baseline(next_generation);

        Ok(previous)
    }

    pub fn compare_candidate_with<CF>(
        &mut self,
        candidate_factory: &CF,
        candidate: &CF::Candidate,
    ) -> BuildCandidatePreparationResult<G, BC, E, A, F, P, Factory, C, CF>
    where
        CF: BuildCandidateFactory<BC::Build>,
        F::Output: Clone,
        C: CalculationOutputComparator<F::Output>,
    {
        let candidate_build = candidate_factory
            .create_candidate(&self.build, candidate)
            .map_err(BuildCandidatePreparationError::Create)?;

        self.compare_candidate_build(&candidate_build)
            .map_err(BuildCandidatePreparationError::Compare)
    }

    fn next_generation(&self) -> Result<CoreGeneration, BuildCalculationCoreMutationError> {
        self.generation
            .checked_next()
            .ok_or(BuildCalculationCoreMutationError::GenerationOverflow)
    }

    fn invalidate_baseline(&mut self, next_generation: CoreGeneration) {
        self.generation = next_generation;
        self.baseline = None;
    }
}
