use crate::{
    effect::{
        EffectEntry, EffectSource, ItemEffectCollectionError, ItemEffectCollector,
        ModifierEffectResolver,
    },
    game::Game,
    item::{ModifierDefinitionProvider, item_instance::ItemInstance},
};

type ItemEffectCollectionResult<P, R, G> = Result<
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
    effects: Vec<EffectEntry<G>>,
}

impl<G> EffectCollection<G>
where
    G: Game,
{
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, EffectEntry<G>> {
        self.effects.iter()
    }

    pub fn into_effects(self) -> Vec<EffectEntry<G>> {
        self.effects
    }

    pub fn collect_from_source<S>(&mut self, source: &S)
    where
        S: EffectSource<G>,
    {
        self.effects.extend(source.collect_effects());
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
        let effects = resolver.resolve_modifier_effects(definition, modifier)?;

        self.effects.extend(effects);

        Ok(())
    }

    pub fn collect_from_item<P, R>(
        &mut self,
        collector: &ItemEffectCollector<'_, P, R>,
        item: &ItemInstance<G>,
    ) -> ItemEffectCollectionResult<P, R, G>
    where
        P: ModifierDefinitionProvider<G>,
        R: ModifierEffectResolver<G>,
    {
        let effects = collector.collect(item)?;

        self.effects.extend(effects);

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
    type Item = &'a EffectEntry<G>;
    type IntoIter = std::slice::Iter<'a, EffectEntry<G>>;

    fn into_iter(self) -> Self::IntoIter {
        self.effects.iter()
    }
}

impl<G> IntoIterator for EffectCollection<G>
where
    G: Game,
{
    type Item = EffectEntry<G>;
    type IntoIter = std::vec::IntoIter<EffectEntry<G>>;

    fn into_iter(self) -> Self::IntoIter {
        self.effects.into_iter()
    }
}
