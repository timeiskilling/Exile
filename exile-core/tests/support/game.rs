#![allow(dead_code)]
use exile_core::game::{Game, ModifierDefinitionIdentity};

use crate::support::TestPassiveNode;

#[derive(Debug)]
pub struct TestGame;

pub struct TestModifierDefinition {
    pub kind: TestModifierKind,
    pub required_item_level: u16,
    pub min_roll: u16,
    pub max_roll: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestModifierKind {
    MovementSpeed,
    MaximumLife,
    AddedPhysicalDamage,
    GrantsPassiveNode { node_id: TestPassiveNodeId },
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestPassiveNodeId {
    ChaosInoculation,
    FullLifeDamage,
    Empty,
}

pub struct TestItemBase {
    pub is_boots: bool,
}

#[derive(Debug, Default, PartialEq)]
pub struct TestItemState {
    pub item_level: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestModifier {
    Rolled { roll: u16 },

    Range { min: u16, max: u16 },

    NoRoll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEffect {
    ChaosImmune,
    SetMaximumLife { value: u32 },
    AddedMaximumLife { amount: u16 },
    IncreasedDamage { percent: u16 },
    IncreasedMovementSpeed { percent: u16 },
    AddedPhysicalDamage { min: u16, max: u16 },
    MinimumMovementSpeed { percent: u16 },
    MaximumMovementSpeed { percent: u16 },
}

#[derive(Debug, PartialEq)]
pub enum TestEffectCondition {
    EnemyOnFullLife,
}

impl ModifierDefinitionIdentity for TestModifierDefinition {
    type Id = TestModifierKind;

    fn modifier_definition_id(&self) -> Self::Id {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestEffectSourceId {
    PassiveNode(TestPassiveNodeId),
    Synthetic(&'static str),
}

impl Game for TestGame {
    type ItemBase = TestItemBase;
    type ItemState = TestItemState;

    type ModifierDefinitionId = TestModifierKind;
    type ModifierDefinition = TestModifierDefinition;
    type ModifierInstance = TestModifier;

    type Effect = TestEffect;
    type EffectCondition = TestEffectCondition;
    type EffectSourceId = TestEffectSourceId;
}
