use std::fmt;

use crate::{game::Game, item::ModifierInstanceId};

#[derive(PartialEq, Eq)]
pub enum EffectOrigin<G>
where
    G: Game,
{
    ItemModifier {
        modifier_instance_id: ModifierInstanceId,
        definition_id: G::ModifierDefinitionId,
    },

    ModifierDefinition {
        definition_id: G::ModifierDefinitionId,
    },

    Source(G::EffectSourceId),
}

impl<G> Clone for EffectOrigin<G>
where
    G: Game,
    G::ModifierDefinitionId: Clone,
    G::EffectSourceId: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::ItemModifier {
                modifier_instance_id,
                definition_id,
            } => Self::ItemModifier {
                modifier_instance_id: *modifier_instance_id,
                definition_id: definition_id.clone(),
            },

            Self::ModifierDefinition { definition_id } => Self::ModifierDefinition {
                definition_id: definition_id.clone(),
            },

            Self::Source(source_id) => Self::Source(source_id.clone()),
        }
    }
}

impl<G> fmt::Debug for EffectOrigin<G>
where
    G: Game,
    G::ModifierDefinitionId: fmt::Debug,
    G::EffectSourceId: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ItemModifier {
                modifier_instance_id,
                definition_id,
            } => formatter
                .debug_struct("ItemModifier")
                .field("modifier_instance_id", modifier_instance_id)
                .field("definition_id", definition_id)
                .finish(),

            Self::ModifierDefinition { definition_id } => formatter
                .debug_struct("ModifierDefinition")
                .field("definition_id", definition_id)
                .finish(),

            Self::Source(source_id) => formatter.debug_tuple("Source").field(source_id).finish(),
        }
    }
}
