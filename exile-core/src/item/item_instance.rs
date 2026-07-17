use std::{fmt, marker::PhantomData};

use crate::{game::Game, item::item_validator::ItemValidator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unvalidated;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validated;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierInstanceId(u64);

#[derive(Debug)]
pub struct StoredModifier<D, M> {
    id: ModifierInstanceId,
    definition_id: D,
    modifier: M,
}

impl<D, M> StoredModifier<D, M> {
    pub fn id(&self) -> ModifierInstanceId {
        self.id
    }

    pub fn definition_id(&self) -> &D {
        &self.definition_id
    }

    pub fn modifier(&self) -> &M {
        &self.modifier
    }

    pub(crate) fn into_modifier(self) -> M {
        self.modifier
    }
}

pub struct ItemInstance<G: Game, ValidationState = Unvalidated> {
    base: G::ItemBase,
    state: G::ItemState,

    modifiers: Vec<StoredModifier<G::ModifierDefinitionId, G::ModifierInstance>>,

    next_modifier_id: u64,
    revision: u64,

    validation_state: PhantomData<ValidationState>,
}

pub struct ItemValidationFailure<G, E>
where
    G: Game,
{
    item: ItemInstance<G, Unvalidated>,
    error: E,
}

impl<G, E> ItemValidationFailure<G, E>
where
    G: Game,
{
    pub fn item(&self) -> &ItemInstance<G, Unvalidated> {
        &self.item
    }

    pub fn error(&self) -> &E {
        &self.error
    }

    pub fn into_item(self) -> ItemInstance<G, Unvalidated> {
        self.item
    }

    pub fn into_error(self) -> E {
        self.error
    }

    pub fn into_parts(self) -> (ItemInstance<G, Unvalidated>, E) {
        (self.item, self.error)
    }
}

impl<G, E> fmt::Debug for ItemValidationFailure<G, E>
where
    G: Game,
    E: fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ItemValidationFailure")
            .field("error", &self.error)
            .finish()
    }
}

impl<G, ValidationState> ItemInstance<G, ValidationState>
where
    G: Game,
{
    pub fn base(&self) -> &G::ItemBase {
        &self.base
    }

    pub fn state(&self) -> &G::ItemState {
        &self.state
    }

    pub fn modifiers(&self) -> &[StoredModifier<G::ModifierDefinitionId, G::ModifierInstance>] {
        &self.modifiers
    }

    pub fn modifier(&self, id: ModifierInstanceId) -> Option<&G::ModifierInstance> {
        self.stored_modifier(id).map(StoredModifier::modifier)
    }

    pub fn stored_modifier(
        &self,
        id: ModifierInstanceId,
    ) -> Option<&StoredModifier<G::ModifierDefinitionId, G::ModifierInstance>> {
        self.modifiers.iter().find(|stored| stored.id == id)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    fn change_validation_state<NextValidationState>(self) -> ItemInstance<G, NextValidationState> {
        ItemInstance {
            base: self.base,
            state: self.state,
            modifiers: self.modifiers,
            next_modifier_id: self.next_modifier_id,
            revision: self.revision,
            validation_state: PhantomData,
        }
    }
}

impl<G> ItemInstance<G, Unvalidated>
where
    G: Game,
{
    pub fn new(base: G::ItemBase, state: G::ItemState) -> Self {
        Self {
            base,
            state,
            modifiers: Vec::new(),
            next_modifier_id: 0,
            revision: 0,
            validation_state: PhantomData,
        }
    }

    pub fn from_parts(
        base: G::ItemBase,
        state: G::ItemState,
        modifiers: Vec<(G::ModifierDefinitionId, G::ModifierInstance)>,
    ) -> Self {
        let mut item = Self::new(base, state);

        for (definition_id, modifier) in modifiers {
            item.push_modifier_unchecked(definition_id, modifier);
        }

        item
    }

    pub fn validate<V>(
        self,
        validator: &V,
    ) -> Result<ItemInstance<G, Validated>, ItemValidationFailure<G, V::Error>>
    where
        V: ItemValidator<G>,
    {
        match validator.validate_item(&self) {
            Ok(()) => Ok(self.change_validation_state()),

            Err(error) => Err(ItemValidationFailure { item: self, error }),
        }
    }

    pub(crate) fn push_modifier_unchecked(
        &mut self,
        definition_id: G::ModifierDefinitionId,
        modifier: G::ModifierInstance,
    ) -> ModifierInstanceId {
        let id = ModifierInstanceId(self.next_modifier_id);

        self.next_modifier_id = self
            .next_modifier_id
            .checked_add(1)
            .expect("modifier instance id overflow");

        self.modifiers.push(StoredModifier {
            id,
            definition_id,
            modifier,
        });

        id
    }
    pub(crate) fn remove_modifier_unchecked(
        &mut self,
        id: ModifierInstanceId,
    ) -> Option<G::ModifierInstance> {
        let index = self.modifiers.iter().position(|stored| stored.id() == id)?;

        let stored = self.modifiers.remove(index);

        Some(stored.into_modifier())
    }

    pub(crate) fn replace_modifier_unchecked(
        &mut self,
        id: ModifierInstanceId,
        definition_id: G::ModifierDefinitionId,
        modifier: G::ModifierInstance,
    ) -> Option<G::ModifierInstance> {
        let stored = self.modifiers.iter_mut().find(|stored| stored.id == id)?;

        stored.definition_id = definition_id;

        let previous = std::mem::replace(&mut stored.modifier, modifier);

        Some(previous)
    }

    pub(crate) fn replace_state_unchecked(&mut self, state: G::ItemState) -> G::ItemState {
        std::mem::replace(&mut self.state, state)
    }

    pub(crate) fn increment_revision(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("item revision overflow");
    }
}

impl<G> ItemInstance<G, Validated>
where
    G: Game,
{
    pub fn into_unvalidated(self) -> ItemInstance<G, Unvalidated> {
        self.change_validation_state()
    }
}
