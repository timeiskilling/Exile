use crate::game::Game;

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

pub struct ItemInstance<G>
where
    G: Game,
{
    base: G::ItemBase,
    state: G::ItemState,

    modifiers: Vec<StoredModifier<G::ModifierDefinitionId, G::ModifierInstance>>,
    next_modifier_id: u64,
    revision: u64,
}

impl<G> ItemInstance<G>
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
        }
    }

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

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn increment_revision(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("item revision overflow");
    }

    pub(crate) fn replace_state_unchecked(&mut self, state: G::ItemState) -> G::ItemState {
        std::mem::replace(&mut self.state, state)
    }
}
