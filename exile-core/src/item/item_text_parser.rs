use crate::item::{game_definition::Game, item_instance::ItemInstance};

pub trait ItemTextParer<G>
where
    G: Game,
{
    type Error;

    fn parse(&self, text: &str) -> Result<ItemInstance<G>, Self::Error>;
}
