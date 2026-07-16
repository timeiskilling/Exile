use crate::game::Game;

pub type ModifierPair<G> = (
    <G as Game>::ModifierDefinitionId,
    <G as Game>::ModifierInstance,
);

pub trait ModifierTextParser<G>
where
    G: Game,
{
    type Error;

    fn try_parse_modifier(&self, line: &str) -> Result<Option<ModifierPair<G>>, Self::Error>;
}
