use std::{marker::PhantomData, mem};

use crate::{
    effect::{
        BuildCalculationRunner, BuildEffectCollector, CalculationBaseline,
        EffectAccumulatorFactory, EffectAccumulatorFinalizer, EffectApplier,
        EffectConditionEvaluator, EffectPlanner,
        calculation::build_calculation_runner::BuildCalculationErrorFor,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CoreGeneration(u64);

impl CoreGeneration {
    fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

pub struct BuildCalculationCore<G, BC, E, A, F, P, Factory>
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
}

impl<G, BC, E, A, F, P, Factory> BuildCalculationCore<G, BC, E, A, F, P, Factory>
where
    G: Game,
    BC: BuildEffectCollector<G>,
    E: EffectConditionEvaluator<G>,
    A: EffectApplier<G>,
    F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
    P: EffectPlanner<G>,
    Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
{
    pub fn new(
        build: BC::Build,
        context: E::Context,
        input: Factory::Input,
        factory: Factory,
        runner: BuildCalculationRunner<BC, E, A, F, P>,
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
        }
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
