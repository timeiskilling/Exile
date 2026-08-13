use crate::{
    item::{
        modifier_definition_provider::Poe2DefinitionRegistryError,
        modifier_validator::Poe2ModifierValidationError,
    },
    repoe_parse::GenerationType,
};

#[derive(Debug, Clone)]
pub enum RuleError {
    InvalidMod,
    InvalidItemRarity,
    AlreadyCraftedMod,
    InvalidModForBase,
    ModNotFound,
    AffixLimitReached(GenerationType),
    AffixLimitsReached,
    FracturedModLimitReached,
    CraftedModLimitReached,
    ValidationError(Poe2ModifierValidationError),
    SameMod,
    RegistryError(Poe2DefinitionRegistryError),
}
