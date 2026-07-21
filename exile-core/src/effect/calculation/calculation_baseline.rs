#[derive(Debug, Clone, PartialEq)]
pub struct CalculationBaseline<R, O> {
    revision: R,
    output: O,
}

impl<R, O> CalculationBaseline<R, O> {
    pub fn new(revision: R, output: O) -> Self {
        Self { revision, output }
    }

    pub fn revision(&self) -> &R {
        &self.revision
    }

    pub fn output(&self) -> &O {
        &self.output
    }

    pub fn into_output(self) -> O {
        self.output
    }

    pub fn into_parts(self) -> (R, O) {
        (self.revision, self.output)
    }
}
