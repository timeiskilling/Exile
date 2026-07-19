use crate::{
    effect::{
        ActiveEffectCollection, SourcedEffectEntry,
        calculation::{
            effect_planning_policy::EffectPlanningPolicy,
            effect_selection_rejection::EffectSelectionRejection,
        },
    },
    game::Game,
};

pub struct EffectExecutionPlan<'a, G>
where
    G: Game,
{
    entries: Vec<&'a SourcedEffectEntry<G>>,
    selection_rejections: Vec<EffectSelectionRejection<'a, G>>,
}

impl<'a, G> EffectExecutionPlan<'a, G>
where
    G: Game,
{
    pub fn build<P>(effects: &ActiveEffectCollection<'a, G>, policy: &P) -> Self
    where
        P: EffectPlanningPolicy<G>,
    {
        let mut entries = effects
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                (
                    policy.phase(entry.effect()),
                    policy.priority(entry.effect()),
                    index,
                    entry,
                )
            })
            .collect::<Vec<_>>();

        entries.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });

        Self {
            entries: entries.into_iter().map(|(_, _, _, entry)| entry).collect(),
            selection_rejections: Vec::new(),
        }
    }

    pub(crate) fn from_entries(
        entries: Vec<&'a SourcedEffectEntry<G>>,
        selection_rejections: Vec<EffectSelectionRejection<'a, G>>,
    ) -> Self {
        Self {
            entries,
            selection_rejections,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Vec<&'a SourcedEffectEntry<G>>,
        Vec<EffectSelectionRejection<'a, G>>,
    ) {
        (self.entries, self.selection_rejections)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn selection_rejections(&self) -> impl Iterator<Item = &EffectSelectionRejection<'a, G>> {
        self.selection_rejections.iter()
    }

    pub fn selection_rejection_count(&self) -> usize {
        self.selection_rejections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &'a SourcedEffectEntry<G>> + '_ {
        self.entries.iter().copied()
    }

    pub fn effects(&self) -> impl Iterator<Item = &'a G::Effect> + '_ {
        self.iter().map(SourcedEffectEntry::effect)
    }
}

impl<'a, G> IntoIterator for EffectExecutionPlan<'a, G>
where
    G: Game,
{
    type Item = &'a SourcedEffectEntry<G>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
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
