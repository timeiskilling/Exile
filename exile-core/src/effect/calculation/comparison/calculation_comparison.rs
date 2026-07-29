use crate::effect::CalculationOutputComparator;

#[derive(Debug, Clone, PartialEq)]
pub struct CalculationComparison<O, D> {
    baseline: O,
    candidate: O,
    difference: D,
}

impl<O, D> CalculationComparison<O, D> {
    pub(crate) fn new(baseline: O, candidate: O, difference: D) -> Self {
        Self {
            baseline,
            candidate,
            difference,
        }
    }

    pub fn between<C>(baseline: O, candidate: O, comparator: &C) -> Self
    where
        C: CalculationOutputComparator<O, Difference = D>,
    {
        let difference = comparator.compare(&baseline, &candidate);

        Self::new(baseline, candidate, difference)
    }

    pub fn baseline(&self) -> &O {
        &self.baseline
    }

    pub fn candidate(&self) -> &O {
        &self.candidate
    }

    pub fn difference(&self) -> &D {
        &self.difference
    }

    pub fn into_parts(self) -> (O, O, D) {
        (self.baseline, self.candidate, self.difference)
    }
}
