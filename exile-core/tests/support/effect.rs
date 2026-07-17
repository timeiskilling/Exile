#![allow(dead_code)]
use std::convert::Infallible;

use exile_core::effect::{
    EffectAccumulatorFactory, EffectAccumulatorFinalizer, EffectApplier, EffectConditionEvaluator,
    EffectEntry, EffectSource, ModifierEffectResolver, PassiveNodeProvider,
    calculation::{EffectConflictKeyResolver, EffectPhaseResolver},
};

use crate::support::{TestEffectSourceId, TestPassiveNodeId};

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

#[derive(Debug, PartialEq, Eq)]
pub struct TestModifierEffectResolver {
    passive_nodes: TestPassiveNodeProvider,
}

impl TestModifierEffectResolver {
    pub fn new(passive_nodes: TestPassiveNodeProvider) -> Self {
        Self { passive_nodes }
    }
}

impl Default for TestModifierEffectResolver {
    fn default() -> Self {
        Self::new(TestPassiveNodeProvider::default())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestEffectResolveError {
    UnsupportedModifier,
    InvalidModifierPayload,

    PassiveNodeProvider(TestPassiveNodeProviderError),
}
impl EffectSource<TestGame> for TestPassiveNode {
    fn effect_source_id(&self) -> TestEffectSourceId {
        let node_id = match self {
            TestPassiveNode::ChaosInoculation => TestPassiveNodeId::ChaosInoculation,

            TestPassiveNode::FullLifeDamage => TestPassiveNodeId::FullLifeDamage,

            TestPassiveNode::Empty => TestPassiveNodeId::Empty,
        };

        TestEffectSourceId::PassiveNode(node_id)
    }

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
            TestModifierKind::MovementSpeed => {
                let TestModifier::Rolled { roll } = modifier else {
                    return Err(TestEffectResolveError::InvalidModifierPayload);
                };

                Ok(vec![EffectEntry::unconditional(
                    TestEffect::IncreasedMovementSpeed { percent: *roll },
                )])
            }

            TestModifierKind::MaximumLife => {
                let TestModifier::Rolled { roll } = modifier else {
                    return Err(TestEffectResolveError::InvalidModifierPayload);
                };

                Ok(vec![EffectEntry::unconditional(
                    TestEffect::AddedMaximumLife { amount: *roll },
                )])
            }

            TestModifierKind::GrantsPassiveNode { node_id } => {
                if !matches!(modifier, TestModifier::NoRoll) {
                    return Err(TestEffectResolveError::InvalidModifierPayload);
                }

                let node = self
                    .passive_nodes
                    .node(&node_id)
                    .map_err(TestEffectResolveError::PassiveNodeProvider)?;

                Ok(node.collect_effects())
            }

            TestModifierKind::Unsupported => Err(TestEffectResolveError::UnsupportedModifier),

            TestModifierKind::AddedPhysicalDamage => {
                let TestModifier::Range { min, max } = modifier else {
                    return Err(TestEffectResolveError::InvalidModifierPayload);
                };

                Ok(vec![EffectEntry::unconditional(
                    TestEffect::AddedPhysicalDamage {
                        min: *min,
                        max: *max,
                    },
                )])
            }
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

    pub added_physical_damage_min: u32,
    pub added_physical_damage_max: u32,
}

impl TestEffectAccumulator {
    pub fn with_base_maximum_life(base_maximum_life: u32) -> Self {
        Self {
            base_maximum_life,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEffectApplyError {
    AddedMaximumLife,
    IncreasedDamage,
    IncreasedMovementSpeed,
    AddedPhysicalDamageMinimum,
    AddedPhysicalDamageMaximum,
}

pub struct TestEffectApplier;

impl EffectApplier<TestGame> for TestEffectApplier {
    type Accumulator = TestEffectAccumulator;
    type Error = TestEffectApplyError;

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
                let next = accumulator
                    .added_maximum_life
                    .checked_add(u32::from(*amount))
                    .ok_or(TestEffectApplyError::AddedMaximumLife)?;

                accumulator.added_maximum_life = next;
            }

            TestEffect::IncreasedDamage { percent } => {
                let next = accumulator
                    .increased_damage_percent
                    .checked_add(u32::from(*percent))
                    .ok_or(TestEffectApplyError::IncreasedDamage)?;

                accumulator.increased_damage_percent = next;
            }

            TestEffect::IncreasedMovementSpeed { percent } => {
                let next = accumulator
                    .increased_movement_speed_percent
                    .checked_add(u32::from(*percent))
                    .ok_or(TestEffectApplyError::IncreasedMovementSpeed)?;

                accumulator.increased_movement_speed_percent = next;
            }

            TestEffect::AddedPhysicalDamage { min, max } => {
                let next_min = accumulator
                    .added_physical_damage_min
                    .checked_add(u32::from(*min))
                    .ok_or(TestEffectApplyError::AddedPhysicalDamageMinimum)?;

                let next_max = accumulator
                    .added_physical_damage_max
                    .checked_add(u32::from(*max))
                    .ok_or(TestEffectApplyError::AddedPhysicalDamageMaximum)?;

                accumulator.added_physical_damage_min = next_min;

                accumulator.added_physical_damage_max = next_max;
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

    pub added_physical_damage_min: u32,
    pub added_physical_damage_max: u32,
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

            added_physical_damage_min: accumulator.added_physical_damage_min,

            added_physical_damage_max: accumulator.added_physical_damage_max,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestCalculationInput {
    pub base_maximum_life: u32,
}

pub struct TestEffectAccumulatorFactory;

impl EffectAccumulatorFactory for TestEffectAccumulatorFactory {
    type Input = TestCalculationInput;
    type Accumulator = TestEffectAccumulator;
    type Error = Infallible;

    fn create(&self, input: &Self::Input) -> Result<Self::Accumulator, Self::Error> {
        Ok(TestEffectAccumulator {
            base_maximum_life: input.base_maximum_life,

            ..TestEffectAccumulator::default()
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestPassiveNodeProviderError {
    NotFound(TestPassiveNodeId),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TestPassiveNodeProvider {
    nodes: Vec<(TestPassiveNodeId, TestPassiveNode)>,
}

impl TestPassiveNodeProvider {
    pub fn new(nodes: Vec<(TestPassiveNodeId, TestPassiveNode)>) -> Self {
        Self { nodes }
    }
}

impl PassiveNodeProvider<TestGame> for TestPassiveNodeProvider {
    type Id = TestPassiveNodeId;
    type Node = TestPassiveNode;
    type Error = TestPassiveNodeProviderError;

    fn node(&self, id: &Self::Id) -> Result<&Self::Node, Self::Error> {
        self.nodes
            .iter()
            .find(|(node_id, _)| node_id == id)
            .map(|(_, node)| node)
            .ok_or(TestPassiveNodeProviderError::NotFound(*id))
    }
}

impl Default for TestPassiveNodeProvider {
    fn default() -> Self {
        Self::new(vec![
            (
                TestPassiveNodeId::ChaosInoculation,
                TestPassiveNode::ChaosInoculation,
            ),
            (
                TestPassiveNodeId::FullLifeDamage,
                TestPassiveNode::FullLifeDamage,
            ),
            (TestPassiveNodeId::Empty, TestPassiveNode::Empty),
        ])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TestEffectPhase {
    Added,
    Increased,
    Final,
}

pub struct TestEffectPhaseResolver;

impl EffectPhaseResolver<TestGame> for TestEffectPhaseResolver {
    type Phase = TestEffectPhase;

    fn phase(&self, effect: &TestEffect) -> Self::Phase {
        match effect {
            TestEffect::AddedMaximumLife { .. } | TestEffect::AddedPhysicalDamage { .. } => {
                TestEffectPhase::Added
            }

            TestEffect::IncreasedDamage { .. } | TestEffect::IncreasedMovementSpeed { .. } => {
                TestEffectPhase::Increased
            }

            TestEffect::ChaosImmune | TestEffect::SetMaximumLife { .. } => TestEffectPhase::Final,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestEffectConflictKey {
    MaximumLifeOverride,
}

pub struct TestEffectConflictKeyResolver;

impl EffectConflictKeyResolver<TestGame> for TestEffectConflictKeyResolver {
    type Key = TestEffectConflictKey;

    fn conflict_key(&self, effect: &TestEffect) -> Option<Self::Key> {
        match effect {
            TestEffect::SetMaximumLife { .. } => Some(TestEffectConflictKey::MaximumLifeOverride),

            _ => None,
        }
    }
}
