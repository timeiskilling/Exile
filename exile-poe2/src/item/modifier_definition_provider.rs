use exile_core::item::ModifierDefinitionProvider;

use crate::item::{
    definition::Poe2DefinitionRegistry,
    state::{Poe2, Poe2ModifierDefinition, Poe2ModifierId},
};

#[derive(Debug, Clone)]
pub enum Poe2DefinitionRegistryError {
    DefinitionNotFound { id: Poe2ModifierId },
}

impl ModifierDefinitionProvider<Poe2> for Poe2DefinitionRegistry {
    type Error = Poe2DefinitionRegistryError;

    fn definition(&self, id: &Poe2ModifierId) -> Result<&Poe2ModifierDefinition, Self::Error> {
        self.definitions
            .get(id)
            .ok_or(Poe2DefinitionRegistryError::DefinitionNotFound { id: *id })
    }
}
