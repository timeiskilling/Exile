use exile_core::item::{ItemRule, ItemValidator, ModifierValidator};

use crate::{
    error::rule_e::RuleError,
    item::{
        affix_state::ItemAffixState, definition::Poe2DefinitionRegistry,
        modifier_validator::Poe2ModifierValidator, state::Poe2,
    },
};

pub struct Poe2ItemRuleValidator<'a> {
    pub provider: &'a Poe2DefinitionRegistry,
}

impl<'a> Poe2ItemRuleValidator<'a> {
    pub fn check_duplicate_modifier(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        target_id: Option<exile_core::item::ModifierInstanceId>,
        definition: &<Poe2 as exile_core::game::Game>::ModifierDefinition,
    ) -> Result<(), RuleError> {
        for stored in item.modifiers() {
            if target_id.is_some_and(|id| id == stored.id()) {
                continue;
            }

            let existing_def = self
                .provider
                .definitions
                .get(stored.definition_id())
                .ok_or(RuleError::ModNotFound)?;

            for group in &definition.groups {
                if existing_def.groups.contains(group) {
                    return Err(RuleError::SameMod);
                }
            }
        }
        Ok(())
    }
}

impl<'a> ItemValidator<Poe2> for Poe2ItemRuleValidator<'a> {
    type Error = RuleError;

    fn validate_item(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
    ) -> Result<(), Self::Error> {
        for stored in item.modifiers() {
            let def = self
                .provider
                .definitions
                .get(stored.definition_id())
                .ok_or(RuleError::ModNotFound)?;

            Poe2ModifierValidator
                .validate_modifier(item, def, stored.modifier())
                .map_err(RuleError::ValidationError)?;
        }

        Ok(())
    }
}

impl<'a> ItemRule<Poe2> for Poe2ItemRuleValidator<'a> {
    type Error = RuleError;

    fn validate_add_modifier(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        definition: &<Poe2 as exile_core::game::Game>::ModifierDefinition,
        modifier: &<Poe2 as exile_core::game::Game>::ModifierInstance,
    ) -> Result<(), Self::Error> {
        let _ = Poe2ModifierValidator
            .validate_modifier(item, definition, modifier)
            .map_err(RuleError::ValidationError);

        self.check_duplicate_modifier(item, None, definition)?;

        ItemAffixState::analyze(item, self.provider)
            .map_err(RuleError::RegistryError)?
            .validate()
    }

    fn validate_replace_modifier(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        target_id: exile_core::item::ModifierInstanceId,
        definition: &<Poe2 as exile_core::game::Game>::ModifierDefinition,
        modifier: &<Poe2 as exile_core::game::Game>::ModifierInstance,
    ) -> Result<(), Self::Error> {
        let _ = Poe2ModifierValidator
            .validate_modifier(item, definition, modifier)
            .map_err(|e| -> RuleError { RuleError::ValidationError(e) });

        self.check_duplicate_modifier(item, Some(target_id), definition)?;

        Ok(())
    }

    fn validate_replace_state(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        new_state: &<Poe2 as exile_core::game::Game>::ItemState,
    ) -> Result<(), Self::Error> {
        if item.state().rarity != new_state.rarity {
            return Err(RuleError::InvalidItemRarity);
        }

        let mock_item =
            exile_core::item::ItemInstance::<Poe2>::new(*item.base(), new_state.clone());

        self.validate_item(&mock_item)?;

        Ok(())
    }

    fn validate_remove_modifier(
        &self,
        item: &exile_core::item::ItemInstance<Poe2, exile_core::item::Unvalidated>,
        id: exile_core::item::ModifierInstanceId,
        _modifier: &<Poe2 as exile_core::game::Game>::ModifierInstance,
    ) -> Result<(), Self::Error> {
        match item.modifier(id) {
            Some(_) => Ok(()),
            None => Err(RuleError::ModNotFound),
        }
    }
}
