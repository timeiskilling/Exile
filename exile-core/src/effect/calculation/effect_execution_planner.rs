use std::hash::Hash;

use crate::{
    effect::{
        ActiveEffectCollection,
        calculation::{
            EffectConflictKeyResolver, EffectExecutionPlan, EffectExecutionPlanSelector,
            EffectExecutionPlanValidationError, EffectExecutionPlanValidator, EffectPhaseResolver,
            EffectPlanner, EffectStrengthResolver,
            effect_priority_resolver::EffectPriorityResolver,
        },
    },
    game::Game,
};

pub struct EffectExecutionPlanner<P, C, R, S> {
    phase_resolver: P,
    plan_validator: EffectExecutionPlanValidator<C>,
    priority_resolver: R,
    plan_selector: EffectExecutionPlanSelector<S>,
}

impl<P, C, R, S> EffectExecutionPlanner<P, C, R, S> {
    pub fn new(
        phase_resolver: P,
        conflict_key_resolver: C,
        priority_resolver: R,
        strength_resolver: S,
    ) -> Self {
        Self {
            phase_resolver,
            plan_validator: EffectExecutionPlanValidator::new(conflict_key_resolver),
            priority_resolver,
            plan_selector: EffectExecutionPlanSelector::new(strength_resolver),
        }
    }
}

impl<G, P, C, R, S> EffectPlanner<G> for EffectExecutionPlanner<P, C, R, S>
where
    G: Game,
    G::ModifierDefinitionId: Clone,
    G::EffectSourceId: Clone,
    P: EffectPhaseResolver<G>,
    C: EffectConflictKeyResolver<G>,
    C::Key: Clone + Eq + Hash,
    R: EffectPriorityResolver<G>,
    S: EffectStrengthResolver<G>,
    S::Key: Eq + Hash,
    S::Strength: Ord,
{
    type Error = EffectExecutionPlanValidationError<G, C::Key>;

    fn plan<'a>(
        &self,
        effects: &ActiveEffectCollection<'a, G>,
    ) -> Result<EffectExecutionPlan<'a, G>, Self::Error> {
        let plan =
            EffectExecutionPlan::build(effects, &self.phase_resolver, &self.priority_resolver);

        self.plan_validator.validate(&plan)?;

        Ok(self.plan_selector.select(plan))
    }
}
