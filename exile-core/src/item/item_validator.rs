use crate::item::{game_definition::Game, item_instance::ItemInstance};

pub trait ItemValidator<G>
where
    G: Game,
{
    type Error;

    fn validate_item(&self, item: &ItemInstance<G>) -> Result<(), Self::Error>;
}
