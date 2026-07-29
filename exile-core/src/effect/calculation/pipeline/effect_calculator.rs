use crate::{
    effect::{
        ActiveEffectCollection, EffectAccumulatorFactory, EffectAccumulatorFinalizer,
        EffectApplier, EffectCollectionApplier,
        calculation::{EffectCalculationOutput, EffectPlanner},
    },
    game::Game,
};

pub type EffectCalculationResult<G, A, F, P> = Result<
    <F as EffectAccumulatorFinalizer>::Output,
    EffectCalculationError<
        <P as EffectPlanner<G>>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

pub type EffectCalculationFromInputResult<G, A, F, P, Factory> = Result<
    <F as EffectAccumulatorFinalizer>::Output,
    EffectCalculationFromInputError<
        <Factory as EffectAccumulatorFactory>::Error,
        <P as EffectPlanner<G>>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

pub type EffectCalculationDetailedResult<'a, G, A, F, P> = Result<
    EffectCalculationOutput<'a, G, <F as EffectAccumulatorFinalizer>::Output>,
    EffectCalculationError<
        <P as EffectPlanner<G>>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

pub type EffectCalculationFromInputDetailedResult<'a, G, A, F, P, Factory> = Result<
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

impl<CreateError, PlanError, ApplyError, FinalizeError>
    From<EffectCalculationError<PlanError, ApplyError, FinalizeError>>
    for EffectCalculationFromInputError<CreateError, PlanError, ApplyError, FinalizeError>
{
    fn from(error: EffectCalculationError<PlanError, ApplyError, FinalizeError>) -> Self {
        match error {
            EffectCalculationError::Plan(error) => Self::Plan(error),

            EffectCalculationError::Apply(error) => Self::Apply(error),

            EffectCalculationError::Finalize(error) => Self::Finalize(error),
        }
    }
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
        accumulator: <A as EffectApplier<G>>::Accumulator,
    ) -> EffectCalculationResult<G, A, F, P>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
    {
        self.calculate_detailed(effects, accumulator)
            .map(|calculation| calculation.into_output())
    }

    pub fn calculate_from_input<G, Factory>(
        &self,
        effects: &ActiveEffectCollection<'_, G>,
        factory: &Factory,
        input: &Factory::Input,
    ) -> EffectCalculationFromInputResult<G, A, F, P, Factory>
    where
        G: Game,
        A: EffectApplier<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
    {
        self.calculate_from_input_detailed(effects, factory, input)
            .map(|calculation| calculation.into_output())
    }

    pub fn calculate_detailed<'a, G>(
        &self,
        effects: &ActiveEffectCollection<'a, G>,
        accumulator: <A as EffectApplier<G>>::Accumulator,
    ) -> EffectCalculationDetailedResult<'a, G, A, F, P>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
    {
        let plan = self
            .planner
            .plan(effects)
            .map_err(EffectCalculationError::Plan)?;

        let accumulator = self
            .collection_applier
            .apply_all_owned(&plan, accumulator)
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
    ) -> EffectCalculationFromInputDetailedResult<'a, G, A, F, P, Factory>
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

        self.calculate_detailed(effects, accumulator)
            .map_err(EffectCalculationFromInputError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::{EffectCalculationError, EffectCalculationFromInputError};

    #[test]
    fn converts_plan_error_into_input_error() {
        let source = EffectCalculationError::<u8, u16, u32>::Plan(10);

        let converted: EffectCalculationFromInputError<(), u8, u16, u32> = source.into();

        assert_eq!(converted, EffectCalculationFromInputError::Plan(10),);
    }

    #[test]
    fn converts_apply_error_into_input_error() {
        let source = EffectCalculationError::<u8, u16, u32>::Apply(20);

        let converted: EffectCalculationFromInputError<(), u8, u16, u32> = source.into();

        assert_eq!(converted, EffectCalculationFromInputError::Apply(20),);
    }

    #[test]
    fn converts_finalize_error_into_input_error() {
        let source = EffectCalculationError::<u8, u16, u32>::Finalize(30);

        let converted: EffectCalculationFromInputError<(), u8, u16, u32> = source.into();

        assert_eq!(converted, EffectCalculationFromInputError::Finalize(30),);
    }
}
