use crate::{
    game::Game,
    item::{ItemInstance, Unvalidated},
};

pub trait ItemTextParser<G>
where
    G: Game,
{
    type Error;

    fn parse(&self, text: &str) -> Result<ItemInstance<G, Unvalidated>, Self::Error>;
}
