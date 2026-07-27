#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatValueDifference<T> {
    baseline: T,
    candidate: T,
}

impl<T> StatValueDifference<T> {
    pub fn between(baseline: T, candidate: T) -> Self {
        Self {
            baseline,
            candidate,
        }
    }

    pub fn baseline(&self) -> &T {
        &self.baseline
    }

    pub fn candidate(&self) -> &T {
        &self.candidate
    }

    pub fn into_parts(self) -> (T, T) {
        (self.baseline, self.candidate)
    }
}

impl<T> StatValueDifference<T>
where
    T: PartialEq,
{
    pub fn is_changed(&self) -> bool {
        self.baseline != self.candidate
    }
}
