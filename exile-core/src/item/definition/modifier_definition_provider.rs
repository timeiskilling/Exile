use crate::game::Game;

pub trait ModifierDefinitionProvider<G>
where
    G: Game,
{
    type Error;

    fn definition(
        &self,
        id: &G::ModifierDefinitionId,
    ) -> Result<&G::ModifierDefinition, Self::Error>;
}
