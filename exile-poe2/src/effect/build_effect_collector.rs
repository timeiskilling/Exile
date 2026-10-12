use ahash::AHashMap;
use exile_core::effect::{BuildEffectCollector, EffectCollection, ItemEffectCollector};
use exile_core::item::{ItemInstance, Validated};

use crate::{
    effect::resolver::Poe2ModifierEffectResolver,
    item::{
        definition::Poe2DefinitionRegistry,
        state::{EquipSlot, Poe2},
    },
};

pub struct Poe2Build {
    pub equipment: AHashMap<EquipSlot, ItemInstance<Poe2, Validated>>,
    // pub passives: Vec<PassiveNodeId>,
    // pub active_skills: Vec<SkillGem>,
}

pub struct Poe2BildEffectCollector<'a> {
    item_collector: ItemEffectCollector<'a, Poe2DefinitionRegistry, Poe2ModifierEffectResolver>,
}

impl<'a> Poe2BildEffectCollector<'a> {
    pub fn new(
        definition_provider: &'a Poe2DefinitionRegistry,
        resolver: &'a Poe2ModifierEffectResolver,
    ) -> Self {
        Self {
            item_collector: ItemEffectCollector::new(definition_provider, resolver),
        }
    }
}

impl BuildEffectCollector<Poe2> for Poe2BildEffectCollector<'_> {
    type Build = Poe2Build;

    type Error = exile_core::effect::ItemEffectCollectionError<
        std::convert::Infallible,
        std::convert::Infallible,
    >;

    fn collect_effects(&self, build: &Self::Build) -> Result<EffectCollection<Poe2>, Self::Error> {
        let mut collection = EffectCollection::new();
        collection
            .collect_from_items(&self.item_collector, build.equipment.values())
            .unwrap();

        Ok(collection)
    }
}
