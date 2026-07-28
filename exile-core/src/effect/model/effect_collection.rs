use crate::{
    effect::{
        EffectOrigin, EffectSource, ItemEffectCollectionError, ItemEffectCollector,
        ModifierEffectResolver, SourcedEffectEntry,
    },
    game::{Game, ModifierDefinitionIdentity},
    item::{ItemInstance, ModifierDefinitionProvider, Validated},
};

pub type ItemEffectCollectionResult<G, P, R> = Result<
    (),
    ItemEffectCollectionError<
        <P as ModifierDefinitionProvider<G>>::Error,
        <R as ModifierEffectResolver<G>>::Error,
    >,
>;

pub struct EffectCollection<G>
where
    G: Game,
{
    entries: Vec<SourcedEffectEntry<G>>,
}

impl<G> EffectCollection<G>
where
    G: Game,
{
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, SourcedEffectEntry<G>> {
        self.entries.iter()
    }

    pub fn into_entries(self) -> Vec<SourcedEffectEntry<G>> {
        self.entries
    }

    pub fn collect_from_source<S>(&mut self, source: &S)
    where
        S: EffectSource<G>,
    {
        for entry in source.collect_effects() {
            self.entries.push(SourcedEffectEntry::new(
                entry,
                EffectOrigin::Source(source.effect_source_id()),
            ));
        }
    }

    pub fn collect_from_sources<'a, I, S>(&mut self, sources: I)
    where
        I: IntoIterator<Item = &'a S>,
        S: EffectSource<G> + 'a,
    {
        for source in sources {
            self.collect_from_source(source);
        }
    }

    pub fn collect_from_modifier<R>(
        &mut self,
        resolver: &R,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<(), R::Error>
    where
        R: ModifierEffectResolver<G>,
    {
        let entries = resolver.resolve_modifier_effects(definition, modifier)?;

        for entry in entries {
            self.entries.push(SourcedEffectEntry::new(
                entry,
                EffectOrigin::ModifierDefinition {
                    definition_id: definition.modifier_definition_id(),
                },
            ));
        }

        Ok(())
    }

    pub fn collect_from_item<P, R>(
        &mut self,
        collector: &ItemEffectCollector<'_, P, R>,
        item: &ItemInstance<G, Validated>,
    ) -> ItemEffectCollectionResult<G, P, R>
    where
        G::ModifierDefinitionId: Clone,
        P: ModifierDefinitionProvider<G>,
        R: ModifierEffectResolver<G>,
    {
        let entries = collector.collect(item)?;

        self.entries.extend(entries);

        Ok(())
    }

    pub fn collect_from_items<'a, I, P, R>(
        &mut self,
        collector: &ItemEffectCollector<'_, P, R>,
        items: I,
    ) -> ItemEffectCollectionResult<G, P, R>
    where
        G: 'a,
        G::ModifierDefinitionId: Clone,
        I: IntoIterator<Item = &'a ItemInstance<G, Validated>>,
        P: ModifierDefinitionProvider<G>,
        R: ModifierEffectResolver<G>,
    {
        let mut collected = EffectCollection::<G>::new();

        for item in items {
            collected.collect_from_item(collector, item)?;
        }

        self.entries.extend(collected.entries);

        Ok(())
    }
}

impl<G> Default for EffectCollection<G>
where
    G: Game,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, G> IntoIterator for &'a EffectCollection<G>
where
    G: Game,
{
    type Item = &'a SourcedEffectEntry<G>;

    type IntoIter = std::slice::Iter<'a, SourcedEffectEntry<G>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl<G> IntoIterator for EffectCollection<G>
where
    G: Game,
{
    type Item = SourcedEffectEntry<G>;

    type IntoIter = std::vec::IntoIter<SourcedEffectEntry<G>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}
