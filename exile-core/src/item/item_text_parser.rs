use crate::{game::Game, item::item_instance::ItemInstance};

pub trait ItemTextParser<G>
where
    G: Game,
{
    type Error;

    fn parse(&self, text: &str) -> Result<ItemInstance<G>, Self::Error>;
}
