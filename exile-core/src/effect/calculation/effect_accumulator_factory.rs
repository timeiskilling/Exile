pub trait EffectAccumulatorFactory {
    type Input;
    type Accumulator;
    type Error;

    fn create(&self, input: &Self::Input) -> Result<Self::Accumulator, Self::Error>;
}
