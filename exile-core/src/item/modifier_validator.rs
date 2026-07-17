use crate::{
    game::Game,
    item::{ItemInstance, Unvalidated},
};

pub trait ModifierValidator<G>
where
    G: Game,
{
    type Error;

    fn validate_modifier(
        &self,
        item: &ItemInstance<G, Unvalidated>,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<(), Self::Error>;
}
