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
            candidate - baseline
        };

        let relative_percent = if baseline == 0.0 {
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
        self.absolute != 0.0
    }

    pub fn is_positive(&self) -> bool {
        self.absolute > 0.0
    }

    pub fn is_negative(&self) -> bool {
        self.absolute < 0.0
    }
}
