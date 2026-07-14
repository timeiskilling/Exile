use std::convert::Infallible;

use exile_core::effect::{
    effect_accumulator_finalizer::EffectAccumulatorFinalizer, effect_applier::EffectApplier,
    effect_condition_evaluator::EffectConditionEvaluator, effect_entry::EffectEntry,
    effect_source::EffectSource, modifier_effect_resolver::ModifierEffectResolver,
};

use super::game::{
    TestEffect, TestEffectCondition, TestGame, TestModifier, TestModifierDefinition,
    TestModifierKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPassiveNode {
    ChaosInoculation,
    FullLifeDamage,
    Empty,
}

#[derive(Debug, PartialEq)]
pub struct TestModifierEffectResolver;

#[derive(Debug, PartialEq)]
pub enum TestEffectResolveError {
    UnsupportedModifier,
}

impl EffectSource<TestGame> for TestPassiveNode {
    fn collect_effects(&self) -> Vec<EffectEntry<TestGame>> {
        match self {
            TestPassiveNode::ChaosInoculation => {
                vec![
                    EffectEntry::unconditional(TestEffect::ChaosImmune),
                    EffectEntry::unconditional(TestEffect::SetMaximumLife { value: 1 }),
                ]
            }

            TestPassiveNode::FullLifeDamage => {
                vec![EffectEntry::conditional(
                    TestEffect::IncreasedDamage { percent: 20 },
                    TestEffectCondition::EnemyOnFullLife,
                )]
            }

            TestPassiveNode::Empty => Vec::new(),
        }
    }
}

impl ModifierEffectResolver<TestGame> for TestModifierEffectResolver {
    type Error = TestEffectResolveError;

    fn resolve_modifier_effects(
        &self,
        definition: &TestModifierDefinition,
        modifier: &TestModifier,
    ) -> Result<Vec<EffectEntry<TestGame>>, Self::Error> {
        match definition.kind {
            TestModifierKind::MovementSpeed => Ok(vec![EffectEntry::unconditional(
                TestEffect::IncreasedMovementSpeed {
                    percent: modifier.roll,
                },
            )]),

            TestModifierKind::MaximumLife => Ok(vec![EffectEntry::unconditional(
                TestEffect::AddedMaximumLife {
                    amount: modifier.roll,
                },
            )]),

            TestModifierKind::Unsupported => Err(TestEffectResolveError::UnsupportedModifier),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TestEffectContext {
    pub enemy_current_life: u32,
    pub enemy_maximum_life: u32,
}

pub struct TestEffectConditionEvaluator;

impl EffectConditionEvaluator<TestGame> for TestEffectConditionEvaluator {
    type Context = TestEffectContext;
    type Error = Infallible;

    fn evaluate_condition(
        &self,
        condition: &TestEffectCondition,
        context: &Self::Context,
    ) -> Result<bool, Self::Error> {
        let result = match condition {
            TestEffectCondition::EnemyOnFullLife => {
                context.enemy_current_life == context.enemy_maximum_life
            }
        };

        Ok(result)
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct TestEffectAccumulator {
    pub base_maximum_life: u32,
    pub added_maximum_life: u32,
    pub maximum_life_override: Option<u32>,

    pub chaos_immune: bool,
    pub increased_damage_percent: u32,
    pub increased_movement_speed_percent: u32,
}

impl TestEffectAccumulator {
    pub fn with_base_maximum_life(base_maximum_life: u32) -> Self {
        Self {
            base_maximum_life,
            ..Self::default()
        }
    }
}

pub struct TestEffectApplier;

impl EffectApplier<TestGame> for TestEffectApplier {
    type Accumulator = TestEffectAccumulator;
    type Error = Infallible;

    fn apply_effect(
        &self,
        effect: &TestEffect,
        accumulator: &mut Self::Accumulator,
    ) -> Result<(), Self::Error> {
        match effect {
            TestEffect::ChaosImmune => {
                accumulator.chaos_immune = true;
            }

            TestEffect::SetMaximumLife { value } => {
                accumulator.maximum_life_override = Some(*value);
            }

            TestEffect::AddedMaximumLife { amount } => {
                accumulator.added_maximum_life += u32::from(*amount);
            }

            TestEffect::IncreasedDamage { percent } => {
                accumulator.increased_damage_percent += u32::from(*percent);
            }

            TestEffect::IncreasedMovementSpeed { percent } => {
                accumulator.increased_movement_speed_percent += u32::from(*percent);
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TestFinalStats {
    pub maximum_life: u32,
    pub chaos_immune: bool,
    pub increased_damage_percent: u32,
    pub increased_movement_speed_percent: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestEffectFinalizeError {
    MaximumLifeOverflow,
}

pub struct TestEffectAccumulatorFinalizer;

impl EffectAccumulatorFinalizer for TestEffectAccumulatorFinalizer {
    type Accumulator = TestEffectAccumulator;
    type Output = TestFinalStats;
    type Error = TestEffectFinalizeError;

    fn finalize(&self, accumulator: Self::Accumulator) -> Result<Self::Output, Self::Error> {
        let maximum_life = match accumulator.maximum_life_override {
            Some(value) => value,

            None => accumulator
                .base_maximum_life
                .checked_add(accumulator.added_maximum_life)
                .ok_or(TestEffectFinalizeError::MaximumLifeOverflow)?,
        };

        Ok(TestFinalStats {
            maximum_life,
            chaos_immune: accumulator.chaos_immune,
            increased_damage_percent: accumulator.increased_damage_percent,
            increased_movement_speed_percent: accumulator.increased_movement_speed_percent,
        })
    }
}
