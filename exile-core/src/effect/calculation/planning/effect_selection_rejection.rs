use crate::{effect::SourcedEffectEntry, game::Game};

pub struct EffectSelectionRejection<'a, G>
where
    G: Game,
{
    rejected: &'a SourcedEffectEntry<G>,
    winner: &'a SourcedEffectEntry<G>,
}

impl<'a, G> EffectSelectionRejection<'a, G>
where
    G: Game,
{
    pub fn new(rejected: &'a SourcedEffectEntry<G>, winner: &'a SourcedEffectEntry<G>) -> Self {
        Self { rejected, winner }
    }

    pub fn rejected(&self) -> &'a SourcedEffectEntry<G> {
        self.rejected
    }

    pub fn winner(&self) -> &'a SourcedEffectEntry<G> {
        self.winner
    }
}
