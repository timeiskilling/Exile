use crate::{game::Game, item::item_instance::ItemInstance};

pub trait ItemValidator<G>
where
    G: Game,
{
    type Error;

    fn validate_item(&self, item: &ItemInstance<G>) -> Result<(), Self::Error>;
}
