use crate::item::{
    game_definition::Game,
    item_instance::{ItemInstance, ModifierInstanceId},
    item_rule::ItemRule,
};

use exile_error::{RemoveModifierError, ReplaceModifierError};

pub struct ItemEditor<R> {
    rules: R,
}

impl<R> ItemEditor<R> {
    pub fn new(rules: R) -> Self {
        Self { rules }
    }

    pub fn add_modifier<G>(
        &self,
        item: &mut ItemInstance<G>,
        definition: &G::ModifierDefinition,
        modifier: G::ModifierInstance,
    ) -> Result<ModifierInstanceId, <R as ItemRule<G>>::Error>
    where
        G: Game,
        R: ItemRule<G>,
    {
        self.rules
            .validate_add_modifier(item, definition, &modifier)?;

        let id = item.push_modifier_unchecked(modifier);

        Ok(id)
    }

    pub fn remove_modifier<G>(
        &self,
        item: &mut ItemInstance<G>,
        id: ModifierInstanceId,
    ) -> Result<G::ModifierInstance, exile_error::RemoveModifierError>
    where
        G: Game,
    {
        item.remove_modifier_unchecked(id)
            .ok_or(RemoveModifierError::ModifierNotFound)
    }

    pub fn replace_modifier<G>(
        &self,
        item: &mut ItemInstance<G>,
        id: ModifierInstanceId,
        definition: &G::ModifierDefinition,
        modifier: G::ModifierInstance,
    ) -> Result<G::ModifierInstance, ReplaceModifierError<<R as ItemRule<G>>::Error>>
    where
        G: Game,
        R: ItemRule<G>,
    {
        if item.modifier(id).is_none() {
            return Err(ReplaceModifierError::ModifierNotFound);
        }

        self.rules
            .validate_replace_modifier(item, id, definition, &modifier)
            .map_err(ReplaceModifierError::Validation)?;

        item.replace_modifier_unchecked(id, modifier)
            .ok_or(ReplaceModifierError::ModifierNotFound)
    }
}
