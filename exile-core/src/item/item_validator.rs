use crate::{
    game::Game,
    item::item_instance::{ItemInstance, Unvalidated},
};

pub trait ItemValidator<G>
where
    G: Game,
{
    type Error;

    fn validate_item(&self, item: &ItemInstance<G, Unvalidated>) -> Result<(), Self::Error>;
}
