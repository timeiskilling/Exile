use crate::{
    effect::{ActiveEffectCollection, SourcedEffectEntry, calculation::EffectPhaseResolver},
    game::Game,
};

pub struct EffectExecutionPlan<'a, G>
where
    G: Game,
{
    entries: Vec<&'a SourcedEffectEntry<G>>,
}

impl<'a, G> EffectExecutionPlan<'a, G>
where
    G: Game,
{
    pub fn build<R>(effects: &ActiveEffectCollection<'a, G>, phase_resolver: &R) -> Self
    where
        R: EffectPhaseResolver<G>,
    {
        let mut entries = effects
            .iter()
            .enumerate()
            .map(|(index, entry)| (phase_resolver.phase(entry.effect()), index, entry))
            .collect::<Vec<_>>();

        entries.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

        Self {
            entries: entries.into_iter().map(|(_, _, entry)| entry).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a SourcedEffectEntry<G>> + '_ {
        self.entries.iter().copied()
    }

    pub fn effects(&self) -> impl Iterator<Item = &'a G::Effect> + '_ {
        self.entries.iter().copied().map(|entry| entry.effect())
    }
}

impl<'a, 'b, G> IntoIterator for &'b EffectExecutionPlan<'a, G>
where
    G: Game,
{
    type Item = &'a SourcedEffectEntry<G>;

    type IntoIter = std::iter::Copied<std::slice::Iter<'b, &'a SourcedEffectEntry<G>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().copied()
    }
}

impl<'a, G> IntoIterator for EffectExecutionPlan<'a, G>
where
    G: Game,
{
    type Item = &'a SourcedEffectEntry<G>;

    type IntoIter = std::vec::IntoIter<&'a SourcedEffectEntry<G>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}
