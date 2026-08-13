use crate::{
    error::rule_e::RuleError,
    item::{
        definition::Poe2DefinitionRegistry,
        modifier_definition_provider::Poe2DefinitionRegistryError,
        state::{ModOrigin, Poe2},
    },
    repoe_parse::GenerationType::{self, Prefix, Suffix},
};
use exile_core::item::ModifierDefinitionProvider;

const MULTIMOD_RUNE_MOD_ID: &str = "local_can_have_additional_crafted_mods";

#[derive(Default)]
pub struct ItemAffixState {
    pub prefixes: u16,
    pub suffixes: u16,
    pub crafted_mods: u16,
    pub fractured_mods: u16,
    pub removable_mods: u16,
    pub has_multimod_rune: bool,
}

impl ItemAffixState {
    pub fn analyze(
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        provider: &Poe2DefinitionRegistry,
    ) -> Result<Self, Poe2DefinitionRegistryError> {
        let mut state = ItemAffixState::default();

        for stored in item.modifiers() {
            let def = provider.definition(stored.definition_id())?;
            let instance = stored.modifier();

            match def.generation_type {
                GenerationType::Prefix => state.prefixes += 1,
                GenerationType::Suffix => state.suffixes += 1,
                _ => {}
            }

            match instance.origin {
                ModOrigin::Crafted => state.crafted_mods += 1,
                ModOrigin::Fractured => state.fractured_mods += 1,
                ModOrigin::Rune if def.id.0 == MULTIMOD_RUNE_MOD_ID => {
                    state.has_multimod_rune = true;
                }
                _ => {}
            }
        }

        Ok(state)
    }

    pub fn validate(&self) -> Result<(), RuleError> {
        if self.prefixes > 3 {
            return Err(RuleError::AffixLimitReached(Prefix));
        }
        if self.suffixes > 3 {
            return Err(RuleError::AffixLimitReached(Suffix));
        }

        if self.prefixes + self.suffixes > 6 {
            return Err(RuleError::AffixLimitsReached);
        }

        if self.fractured_mods > 1 {
            return Err(RuleError::FracturedModLimitReached);
        }

        let max_crafted = if self.has_multimod_rune { 2 } else { 1 };

        if self.crafted_mods > max_crafted {
            return Err(RuleError::CraftedModLimitReached);
        }

        Ok(())
    }
}
