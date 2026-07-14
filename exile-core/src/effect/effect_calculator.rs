use crate::{
    effect::{
        active_effect_collection::ActiveEffectCollection,
        effect_accumulator_finalizer::EffectAccumulatorFinalizer, effect_applier::EffectApplier,
        effect_collection_applier::EffectCollectionApplier,
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

#[derive(Debug, PartialEq, Eq)]
pub enum EffectCalculationError<ApplyError, FinalizeError> {
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
}
