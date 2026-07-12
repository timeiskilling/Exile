use crate::item::{
    game_definition::Game,
    item_instance::{ItemInstance, ModifierInstanceId},
};

pub trait ItemRule<G>
where
    G: Game,
{
    type Error;

    fn validate_add_modifier(
        &self,
        item: &ItemInstance<G>,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<(), Self::Error>;

    fn validate_replace_modifier(
        &self,
        item: &ItemInstance<G>,
        target_id: ModifierInstanceId,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<(), Self::Error>;
}
