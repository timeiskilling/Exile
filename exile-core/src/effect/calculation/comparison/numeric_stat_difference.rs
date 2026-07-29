#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericStatDifference {
    baseline: f64,
    candidate: f64,
    absolute: f64,
    relative_percent: Option<f64>,
}

impl NumericStatDifference {
    pub fn between(baseline: f64, candidate: f64) -> Self {
        let absolute = if baseline == candidate {
            0.0
        } else {
            let difference = candidate - baseline;

            if difference == 0.0 { 0.0 } else { difference }
        };

        let relative_percent = if baseline == 0.0
            || !baseline.is_finite()
            || !candidate.is_finite()
            || !absolute.is_finite()
        {
            None
        } else {
            Some(absolute / baseline * 100.0)
        };

        Self {
            baseline,
            candidate,
            absolute,
            relative_percent,
        }
    }

    pub fn baseline(&self) -> f64 {
        self.baseline
    }

    pub fn candidate(&self) -> f64 {
        self.candidate
    }

    pub fn absolute(&self) -> f64 {
        self.absolute
    }

    pub fn relative_percent(&self) -> Option<f64> {
        self.relative_percent
    }

    pub fn is_changed(&self) -> bool {
        self.baseline != self.candidate
    }

    pub fn is_positive(&self) -> bool {
        self.absolute > 0.0
    }

    pub fn is_negative(&self) -> bool {
        self.absolute < 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::NumericStatDifference;

    #[test]
    fn calculates_finite_difference() {
        let difference = NumericStatDifference::between(100.0, 125.0);

        assert_eq!(difference.baseline(), 100.0,);

        assert_eq!(difference.candidate(), 125.0,);

        assert_eq!(difference.absolute(), 25.0,);

        assert_eq!(difference.relative_percent(), Some(25.0),);

        assert!(difference.is_changed());
        assert!(difference.is_positive());
        assert!(!difference.is_negative());
    }

    #[test]
    fn normalizes_negative_zero() {
        let difference = NumericStatDifference::between(0.0, -0.0);

        assert_eq!(difference.absolute().to_bits(), 0.0_f64.to_bits(),);

        assert_eq!(difference.relative_percent(), None,);

        assert!(!difference.is_changed());
    }

    #[test]
    fn omits_relative_percentage_for_infinity() {
        let difference = NumericStatDifference::between(100.0, f64::INFINITY);

        assert_eq!(difference.absolute(), f64::INFINITY,);

        assert_eq!(difference.relative_percent(), None,);

        assert!(difference.is_changed());
        assert!(difference.is_positive());
    }

    #[test]
    fn omits_relative_percentage_for_nan() {
        let difference = NumericStatDifference::between(f64::NAN, 100.0);

        assert!(difference.absolute().is_nan(),);

        assert_eq!(difference.relative_percent(), None,);

        assert!(difference.is_changed());
        assert!(!difference.is_positive());
        assert!(!difference.is_negative());
    }
}
