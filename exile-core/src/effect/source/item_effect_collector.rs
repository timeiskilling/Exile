use crate::{
    effect::{EffectOrigin, ModifierEffectResolver, SourcedEffectEntry},
    game::Game,
    item::{ItemInstance, ModifierDefinitionProvider, Validated},
};

pub type ItemEffectCollectorResult<G, P, R> = Result<
    Vec<SourcedEffectEntry<G>>,
    ItemEffectCollectionError<
        <P as ModifierDefinitionProvider<G>>::Error,
        <R as ModifierEffectResolver<G>>::Error,
    >,
>;

#[derive(Debug, PartialEq, Eq)]
pub enum ItemEffectCollectionError<DefinitionError, ResolveError> {
    DefinitionProvider(DefinitionError),
    Resolver(ResolveError),
}

pub struct ItemEffectCollector<'a, P, R> {
    definition_provider: &'a P,
    resolver: &'a R,
}

impl<'a, P, R> ItemEffectCollector<'a, P, R> {
    pub fn new(definition_provider: &'a P, resolver: &'a R) -> Self {
        Self {
            definition_provider,
            resolver,
        }
    }

    pub fn collect<G>(
        &self,
        item: &ItemInstance<G, Validated>,
    ) -> ItemEffectCollectorResult<G, P, R>
    where
        G: Game,
        G::ModifierDefinitionId: Clone,
        P: ModifierDefinitionProvider<G>,
        R: ModifierEffectResolver<G>,
    {
        let mut effects = Vec::new();

        for stored_modifier in item.modifiers() {
            let definition = self
                .definition_provider
                .definition(stored_modifier.definition_id())
                .map_err(ItemEffectCollectionError::DefinitionProvider)?;

            let modifier_effects = self
                .resolver
                .resolve_modifier_effects(definition, stored_modifier.modifier())
                .map_err(ItemEffectCollectionError::Resolver)?;

            for entry in modifier_effects {
                effects.push(SourcedEffectEntry::new(
                    entry,
                    EffectOrigin::ItemModifier {
                        modifier_instance_id: stored_modifier.id(),

                        definition_id: stored_modifier.definition_id().clone(),
                    },
                ));
            }
        }

        Ok(effects)
    }
}
