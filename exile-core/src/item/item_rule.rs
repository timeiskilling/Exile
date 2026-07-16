use crate::{
    game::Game,
    item::item_instance::{ItemInstance, ModifierInstanceId, Unvalidated},
};

pub trait ItemRule<G>
where
    G: Game,
{
    type Error;

    fn validate_add_modifier(
        &self,
        item: &ItemInstance<G, Unvalidated>,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<(), Self::Error>;

    fn validate_replace_modifier(
        &self,
        item: &ItemInstance<G, Unvalidated>,
        target_id: ModifierInstanceId,
        definition: &G::ModifierDefinition,
        modifier: &G::ModifierInstance,
    ) -> Result<(), Self::Error>;

    fn validate_replace_state(
        &self,
        item: &ItemInstance<G, Unvalidated>,
        new_state: &G::ItemState,
    ) -> Result<(), Self::Error>;

    fn validate_remove_modifier(
        &self,
        item: &ItemInstance<G, Unvalidated>,
        id: ModifierInstanceId,
        modifier: &G::ModifierInstance,
    ) -> Result<(), Self::Error>;
}
