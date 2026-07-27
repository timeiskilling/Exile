use crate::{effect::calculation::EffectExecutionPlan, game::Game};

pub struct EffectCalculationOutput<'a, G, O>
where
    G: Game,
{
    output: O,
    execution_plan: EffectExecutionPlan<'a, G>,
}

impl<'a, G, O> EffectCalculationOutput<'a, G, O>
where
    G: Game,
{
    pub fn new(output: O, execution_plan: EffectExecutionPlan<'a, G>) -> Self {
        Self {
            output,
            execution_plan,
        }
    }

    pub fn output(&self) -> &O {
        &self.output
    }

    pub fn execution_plan(&self) -> &EffectExecutionPlan<'a, G> {
        &self.execution_plan
    }

    pub fn into_output(self) -> O {
        self.output
    }

    pub fn into_parts(self) -> (O, EffectExecutionPlan<'a, G>) {
        (self.output, self.execution_plan)
    }
}
