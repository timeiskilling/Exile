pub trait CalculationOutputComparator<O> {
    type Difference;

    fn compare(&self, baseline: &O, candidate: &O) -> Self::Difference;
}
