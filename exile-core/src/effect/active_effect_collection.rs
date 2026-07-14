use crate::{effect::effect_entry::EffectEntry, game::Game};

pub struct ActiveEffectCollection<'a, G>
where
    G: Game,
{
    entries: Vec<&'a EffectEntry<G>>,
}

impl<'a, G> ActiveEffectCollection<'a, G>
where
    G: Game,
{
    pub(crate) fn new(entries: Vec<&'a EffectEntry<G>>) -> Self {
        Self { entries }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a EffectEntry<G>> + '_ {
        self.entries.iter().copied()
    }

    pub fn effects(&self) -> impl Iterator<Item = &'a G::Effect> + '_ {
        self.entries.iter().copied().map(|entry| entry.effect())
    }
}

impl<'a, 'b, G> IntoIterator for &'b ActiveEffectCollection<'a, G>
where
    G: Game,
{
    type Item = &'a EffectEntry<G>;

    type IntoIter = std::iter::Copied<std::slice::Iter<'b, &'a EffectEntry<G>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter().copied()
    }
}

impl<'a, G> IntoIterator for ActiveEffectCollection<'a, G>
where
    G: Game,
{
    type Item = &'a EffectEntry<G>;
    type IntoIter = std::vec::IntoIter<&'a EffectEntry<G>>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}
