use crate::{
    effect::{
        ActiveEffectCollection, EffectAccumulatorFactory, EffectAccumulatorFinalizer,
        EffectApplier, EffectCollectionApplier,
        calculation::{EffectCalculationOutput, EffectPlanner},
    },
    game::Game,
};

type EffectCalculationResult<G, A, F, P> = Result<
    <F as EffectAccumulatorFinalizer>::Output,
    EffectCalculationError<
        <P as EffectPlanner<G>>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

type EffectCalculationFromInputResult<G, A, F, P, Factory> = Result<
    <F as EffectAccumulatorFinalizer>::Output,
    EffectCalculationFromInputError<
        <Factory as EffectAccumulatorFactory>::Error,
        <P as EffectPlanner<G>>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

type EffectCalculationResultFromInput<'a, A, G, F, P> = Result<
    EffectCalculationOutput<'a, G, <F as EffectAccumulatorFinalizer>::Output>,
    EffectCalculationError<
        <P as EffectPlanner<G>>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

type EffectCalculationFromInputDetailed<'a, G, A, F, P, Factory> = Result<
    EffectCalculationOutput<'a, G, <F as EffectAccumulatorFinalizer>::Output>,
    EffectCalculationFromInputError<
        <Factory as EffectAccumulatorFactory>::Error,
        <P as EffectPlanner<G>>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

#[derive(Debug, PartialEq, Eq)]
pub enum EffectCalculationError<PlanError, ApplyError, FinalizeError> {
    Plan(PlanError),
    Apply(ApplyError),
    Finalize(FinalizeError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EffectCalculationFromInputError<CreateError, PlanError, ApplyError, FinalizeError> {
    CreateAccumulator(CreateError),
    Plan(PlanError),
    Apply(ApplyError),
    Finalize(FinalizeError),
}

pub struct EffectCalculator<A, F, P> {
    collection_applier: EffectCollectionApplier<A>,
    finalizer: F,
    planner: P,
}

impl<A, F, P> EffectCalculator<A, F, P> {
    pub fn new(effect_applier: A, finalizer: F, planner: P) -> Self {
        Self {
            collection_applier: EffectCollectionApplier::new(effect_applier),
            finalizer,
            planner,
        }
    }

    pub fn calculate<G>(
        &self,
        effects: &ActiveEffectCollection<'_, G>,
        mut accumulator: <A as EffectApplier<G>>::Accumulator,
    ) -> EffectCalculationResult<G, A, F, P>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = <A as EffectApplier<G>>::Accumulator>,
        P: EffectPlanner<G>,
    {
        let plan = self
            .planner
            .plan(effects)
            .map_err(EffectCalculationError::Plan)?;

        self.collection_applier
            .apply_all(&plan, &mut accumulator)
            .map_err(EffectCalculationError::Apply)?;

        self.finalizer
            .finalize(accumulator)
            .map_err(EffectCalculationError::Finalize)
    }

    pub fn calculate_from_input<G, Factory>(
        &self,
        effects: &ActiveEffectCollection<'_, G>,
        factory: &Factory,
        input: &<Factory as EffectAccumulatorFactory>::Input,
    ) -> EffectCalculationFromInputResult<G, A, F, P, Factory>
    where
        G: Game,
        A: EffectApplier<G>,
        Factory: EffectAccumulatorFactory<Accumulator = <A as EffectApplier<G>>::Accumulator>,
        F: EffectAccumulatorFinalizer<Accumulator = <A as EffectApplier<G>>::Accumulator>,
        P: EffectPlanner<G>,
    {
        let accumulator = factory
            .create(input)
            .map_err(EffectCalculationFromInputError::CreateAccumulator)?;

        match self.calculate(effects, accumulator) {
            Ok(output) => Ok(output),

            Err(EffectCalculationError::Plan(error)) => {
                Err(EffectCalculationFromInputError::Plan(error))
            }

            Err(EffectCalculationError::Apply(error)) => {
                Err(EffectCalculationFromInputError::Apply(error))
            }

            Err(EffectCalculationError::Finalize(error)) => {
                Err(EffectCalculationFromInputError::Finalize(error))
            }
        }
    }

    pub fn calculate_detailed<'a, G>(
        &self,
        effects: &ActiveEffectCollection<'a, G>,
        mut accumulator: <A as EffectApplier<G>>::Accumulator,
    ) -> EffectCalculationResultFromInput<'a, A, G, F, P>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = <A as EffectApplier<G>>::Accumulator>,
        P: EffectPlanner<G>,
    {
        let plan = self
            .planner
            .plan(effects)
            .map_err(EffectCalculationError::Plan)?;

        self.collection_applier
            .apply_all(&plan, &mut accumulator)
            .map_err(EffectCalculationError::Apply)?;

        let output = self
            .finalizer
            .finalize(accumulator)
            .map_err(EffectCalculationError::Finalize)?;

        Ok(EffectCalculationOutput::new(output, plan))
    }

    pub fn calculate_from_input_detailed<'a, G, Factory>(
        &self,
        effects: &ActiveEffectCollection<'a, G>,
        factory: &Factory,
        input: &Factory::Input,
    ) -> EffectCalculationFromInputDetailed<'a, G, A, F, P, Factory>
    where
        G: Game,
        A: EffectApplier<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
    {
        let accumulator = factory
            .create(input)
            .map_err(EffectCalculationFromInputError::CreateAccumulator)?;

        match self.calculate_detailed(effects, accumulator) {
            Ok(calculation) => Ok(calculation),

            Err(EffectCalculationError::Plan(error)) => {
                Err(EffectCalculationFromInputError::Plan(error))
            }

            Err(EffectCalculationError::Apply(error)) => {
                Err(EffectCalculationFromInputError::Apply(error))
            }

            Err(EffectCalculationError::Finalize(error)) => {
                Err(EffectCalculationFromInputError::Finalize(error))
            }
        }
    }
}
