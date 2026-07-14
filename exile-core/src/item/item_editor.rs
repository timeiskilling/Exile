use exile_error::{RemoveModifierError, ReplaceModifierError};

use crate::{
    game::Game,
    item::{
        item_instance::{ItemInstance, ModifierInstanceId},
        item_rule::ItemRule,
    },
};

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
        item.increment_revision();

        Ok(id)
    }

    pub fn remove_modifier<G>(
        &self,
        item: &mut ItemInstance<G>,
        id: ModifierInstanceId,
    ) -> Result<G::ModifierInstance, RemoveModifierError<<R as ItemRule<G>>::Error>>
    where
        G: Game,
        R: ItemRule<G>,
    {
        {
            let modifier = item
                .modifier(id)
                .ok_or(RemoveModifierError::ModifierNotFound)?;

            self.rules
                .validate_remove_modifier(item, id, modifier)
                .map_err(RemoveModifierError::Validation)?;
        }

        let removed = item
            .remove_modifier_unchecked(id)
            .expect("modifier existed before remove validation");

        item.increment_revision();

        Ok(removed)
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

        let previous = item
            .replace_modifier_unchecked(id, modifier)
            .expect("modifier existed before replace validation");

        item.increment_revision();

        Ok(previous)
    }

    pub fn replace_state<G>(
        &self,
        item: &mut ItemInstance<G>,
        new_state: G::ItemState,
    ) -> Result<G::ItemState, <R as ItemRule<G>>::Error>
    where
        G: Game,
        R: ItemRule<G>,
    {
        self.rules.validate_replace_state(item, &new_state)?;

        let previous_state = item.replace_state_unchecked(new_state);

        item.increment_revision();

        Ok(previous_state)
    }
}
