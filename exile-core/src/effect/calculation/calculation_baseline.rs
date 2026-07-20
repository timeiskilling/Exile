#[derive(Debug, Clone, PartialEq)]
pub struct CalculationBaseline<O> {
    output: O,
}

impl<O> CalculationBaseline<O> {
    pub fn new(output: O) -> Self {
        Self { output }
    }

    pub fn output(&self) -> &O {
        &self.output
    }

    pub fn into_output(self) -> O {
        self.output
    }
}
