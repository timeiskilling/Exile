pub trait EffectAccumulatorFinalizer {
    type Accumulator;
    type Output;
    type Error;

    fn finalize(&self, accumulator: Self::Accumulator) -> Result<Self::Output, Self::Error>;
}
