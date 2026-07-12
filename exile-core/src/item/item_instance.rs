use super::game_definition::Game;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModifierInstanceId(u64);

#[derive(Debug)]
pub struct StoredModifier<M> {
    id: ModifierInstanceId,
    modifier: M,
}

impl<M> StoredModifier<M> {
    pub fn id(&self) -> ModifierInstanceId {
        self.id
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

    modifiers: Vec<StoredModifier<G::ModifierInstance>>,
    next_modifier_id: u64,
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
        }
    }

    pub fn base(&self) -> &G::ItemBase {
        &self.base
    }

    pub fn state(&self) -> &G::ItemState {
        &self.state
    }

    pub fn modifiers(&self) -> &[StoredModifier<G::ModifierInstance>] {
        &self.modifiers
    }

    pub fn modifier(&self, id: ModifierInstanceId) -> Option<&G::ModifierInstance> {
        self.modifiers
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| &entry.modifier)
    }

    pub(crate) fn push_modifier_unchecked(
        &mut self,
        modifier: G::ModifierInstance,
    ) -> ModifierInstanceId {
        let id = ModifierInstanceId(self.next_modifier_id);

        self.next_modifier_id += 1;

        self.modifiers.push(StoredModifier { id, modifier });

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
        modifier: G::ModifierInstance,
    ) -> Option<G::ModifierInstance> {
        let stored = self.modifiers.iter_mut().find(|stored| stored.id == id)?;
        let previous = std::mem::replace(&mut stored.modifier, modifier);

        Some(previous)
    }
}
