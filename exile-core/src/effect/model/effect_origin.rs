use crate::{game::Game, item::ModifierInstanceId};

#[derive(Debug, PartialEq, Eq)]
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
