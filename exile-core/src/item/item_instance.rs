use std::marker::PhantomData;

use crate::{game::Game, item::item_validator::ItemValidator};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unvalidated;

/// Предмет успішно пройшов повну domain-validation.
///
/// Такий предмет можна передавати в effect/calculation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Validated;

/// Стабільний runtime ID конкретного modifier instance.
///
/// Це не ID definition.
/// Два modifiers однієї definition матимуть різні ModifierInstanceId.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierInstanceId(u64);

/// Modifier, який зберігається всередині предмета.
///
/// `definition_id` вказує, якою definition описується modifier.
/// `modifier` зберігає конкретний runtime payload.
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

/// Runtime instance предмета.
///
/// `ValidationState` існує лише на рівні типів.
/// `PhantomData` не додає runtime-даних до структури.
///
/// Default state — `Unvalidated`, тому:
///
/// ```text
/// ItemInstance<G>
/// ```
///
/// означає:
///
/// ```text
/// ItemInstance<G, Unvalidated>
/// ```
pub struct ItemInstance<G: Game, ValidationState = Unvalidated> {
    base: G::ItemBase,
    state: G::ItemState,

    modifiers: Vec<StoredModifier<G::ModifierDefinitionId, G::ModifierInstance>>,

    next_modifier_id: u64,
    revision: u64,

    validation_state: PhantomData<ValidationState>,
}

/// Методи читання, доступні для будь-якого validation state.
///
/// Вони працюють і з:
///
/// ```text
/// ItemInstance<G, Unvalidated>
/// ItemInstance<G, Validated>
/// ```
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

    /// Змінює лише compile-time validation marker.
    ///
    /// Всі runtime-дані переміщуються без clone.
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

/// Створення, validation і мутації неперевіреного предмета.
///
/// Unchecked methods навмисно недоступні для:
///
/// ```text
/// ItemInstance<G, Validated>
/// ```
impl<G> ItemInstance<G, Unvalidated>
where
    G: Game,
{
    /// Створює новий порожній неперевірений предмет.
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

    /// Створює неперевірений snapshot із готових частин.
    ///
    /// Призначений для:
    ///
    /// - text parser;
    /// - deserialization;
    /// - import;
    /// - migration.
    ///
    /// Метод не виконує domain-validation.
    /// Кожному modifier автоматично призначається runtime ID.
    /// Revision залишається рівною нулю.
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

    /// Перевіряє предмет і при успіху переводить його
    /// у compile-time стан `Validated`.
    ///
    /// При помилці `self` буде спожитий.
    pub fn validate<V>(self, validator: &V) -> Result<ItemInstance<G, Validated>, V::Error>
    where
        V: ItemValidator<G>,
    {
        validator.validate_item(&self)?;

        Ok(self.change_validation_state())
    }

    /// Додає modifier без domain-validation.
    ///
    /// Повинен викликатися лише кодом crate, наприклад ItemEditor.
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

    /// Видаляє modifier без domain-validation.
    pub(crate) fn remove_modifier_unchecked(
        &mut self,
        id: ModifierInstanceId,
    ) -> Option<G::ModifierInstance> {
        let index = self.modifiers.iter().position(|stored| stored.id() == id)?;

        let stored = self.modifiers.remove(index);

        Some(stored.into_modifier())
    }

    /// Замінює definition ID і modifier payload.
    ///
    /// Повертає попередній modifier instance.
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

    /// Замінює state без domain-validation.
    ///
    /// Повертає попередній state.
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
