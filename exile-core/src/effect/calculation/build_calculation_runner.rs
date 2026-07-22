use crate::{
    effect::{
        BuildEffectCollector, EffectAccumulatorFactory, EffectAccumulatorFinalizer, EffectApplier,
        EffectCalculationFromInputError, EffectCalculator, EffectCollectionEvaluator,
        EffectConditionEvaluator, EffectPlanner,
    },
    game::Game,
};

#[derive(Debug)]
pub enum BuildCalculationError<CollectError, EvaluateError, CalculateError> {
    Collect(CollectError),
    Evaluate(EvaluateError),
    Calculate(CalculateError),
}

pub type BuildCalculationOutput<F> = <F as EffectAccumulatorFinalizer>::Output;

pub type BuildEffectCalculationError<G, A, F, P, Factory> = EffectCalculationFromInputError<
    <Factory as EffectAccumulatorFactory>::Error,
    <P as EffectPlanner<G>>::Error,
    <A as EffectApplier<G>>::Error,
    <F as EffectAccumulatorFinalizer>::Error,
>;

pub type BuildCalculationErrorFor<G, BC, E, A, F, P, Factory> = BuildCalculationError<
    <BC as BuildEffectCollector<G>>::Error,
    <E as EffectConditionEvaluator<G>>::Error,
    BuildEffectCalculationError<G, A, F, P, Factory>,
>;

pub type BuildCalculationResult<G, BC, E, A, F, P, Factory> =
    Result<BuildCalculationOutput<F>, BuildCalculationErrorFor<G, BC, E, A, F, P, Factory>>;

pub struct BuildCalculationRunner<BC, E, A, F, P> {
    build_collector: BC,
    evaluator: EffectCollectionEvaluator<E>,
    calculator: EffectCalculator<A, F, P>,
}

impl<BC, E, A, F, P> BuildCalculationRunner<BC, E, A, F, P> {
    pub fn new(
        build_collector: BC,
        evaluator: EffectCollectionEvaluator<E>,
        calculator: EffectCalculator<A, F, P>,
    ) -> Self {
        Self {
            build_collector,
            evaluator,
            calculator,
        }
    }

    pub fn build_collector(&self) -> &BC {
        &self.build_collector
    }

    pub fn evaluator(&self) -> &EffectCollectionEvaluator<E> {
        &self.evaluator
    }

    pub fn calculator(&self) -> &EffectCalculator<A, F, P> {
        &self.calculator
    }

    pub fn into_parts(self) -> (BC, EffectCollectionEvaluator<E>, EffectCalculator<A, F, P>) {
        (self.build_collector, self.evaluator, self.calculator)
    }

    pub fn calculate_build<G, Factory>(
        &self,
        build: &<BC as BuildEffectCollector<G>>::Build,
        context: &<E as EffectConditionEvaluator<G>>::Context,
        factory: &Factory,
        input: &Factory::Input,
    ) -> BuildCalculationResult<G, BC, E, A, F, P, Factory>
    where
        G: Game,
        BC: BuildEffectCollector<G>,
        E: EffectConditionEvaluator<G>,
        A: EffectApplier<G>,
        F: EffectAccumulatorFinalizer<Accumulator = A::Accumulator>,
        P: EffectPlanner<G>,
        Factory: EffectAccumulatorFactory<Accumulator = A::Accumulator>,
    {
        let effects = self
            .build_collector
            .collect_effects(build)
            .map_err(BuildCalculationError::Collect)?;

        let active_effects = self
            .evaluator
            .collect_active(&effects, context)
            .map_err(BuildCalculationError::Evaluate)?;

        self.calculator
            .calculate_from_input(&active_effects, factory, input)
            .map_err(BuildCalculationError::Calculate)
    }
}
