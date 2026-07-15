use crate::{
    effect::{
        ActiveEffectCollection, EffectAccumulatorFactory, EffectAccumulatorFinalizer,
        EffectApplier, EffectCollectionApplier,
    },
    game::Game,
};

type EffectCalculationResult<G, A, F> = Result<
    <F as EffectAccumulatorFinalizer>::Output,
    EffectCalculationError<
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

type EffectCalculationFromInputResult<G, A, F, Factory> = Result<
    <F as EffectAccumulatorFinalizer>::Output,
    EffectCalculationFromInputError<
        <Factory as EffectAccumulatorFactory>::Error,
        <A as EffectApplier<G>>::Error,
        <F as EffectAccumulatorFinalizer>::Error,
    >,
>;

#[derive(Debug, PartialEq, Eq)]
pub enum EffectCalculationError<ApplyError, FinalizeError> {
    Apply(ApplyError),
    Finalize(FinalizeError),
}

#[derive(Debug, PartialEq, Eq)]
pub enum EffectCalculationFromInputError<CreateError, ApplyError, FinalizeError> {
    CreateAccumulator(CreateError),
    Apply(ApplyError),
    Finalize(FinalizeError),
}

pub struct EffectCalculator<A, F> {
    collection_applier: EffectCollectionApplier<A>,
    finalizer: F,
}

impl<A, F> EffectCalculator<A, F> {
    pub fn new(effect_applier: A, finalizer: F) -> Self {
        Self {
            collection_applier: EffectCollectionApplier::new(effect_applier),
            finalizer,
        }
    }

    pub fn calculate<G>(
        &self,
        effects: &ActiveEffectCollection<'_, G>,
        mut accumulator: <A as EffectApplier<G>>::Accumulator,
    ) -> EffectCalculationResult<G, A, F>
    where
        G: Game,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = <A as EffectApplier<G>>::Accumulator>,
    {
        self.collection_applier
            .apply_all(effects, &mut accumulator)
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
    ) -> EffectCalculationFromInputResult<G, A, F, Factory>
    where
        G: Game,
        A: EffectApplier<G>,
        Factory: EffectAccumulatorFactory<Accumulator = <A as EffectApplier<G>>::Accumulator>,
        F: EffectAccumulatorFinalizer<Accumulator = <A as EffectApplier<G>>::Accumulator>,
    {
        let accumulator = factory
            .create(input)
            .map_err(EffectCalculationFromInputError::CreateAccumulator)?;

        match self.calculate(effects, accumulator) {
            Ok(output) => Ok(output),

            Err(EffectCalculationError::Apply(error)) => {
                Err(EffectCalculationFromInputError::Apply(error))
            }

            Err(EffectCalculationError::Finalize(error)) => {
                Err(EffectCalculationFromInputError::Finalize(error))
            }
        }
    }
}
