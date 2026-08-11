use exile_core::game::{Game, ModifierDefinitionIdentity};
use exile_core::item::{ItemInstance, ModifierDefinitionProvider, Unvalidated};
use std::collections::HashMap;

use crate::repoe_parse::{
    GenerationType, ItemClass, Properties, RawModsFile, Requirements, parse_mods_json,
    read_json_file,
};

pub struct Poe2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemRarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

#[derive(Debug, Clone)]
pub struct Poe2ItemState {
    pub item_level: u32,
    pub quality: u32,
    pub rarity: ItemRarity,
    pub is_corrupted: bool,

    pub base_name: String,
    pub drop_level: u32,
    pub properties: Properties,
    pub requirements: Option<Requirements>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Poe2ModifierId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Poe2ModifierKind(pub String);

#[derive(Debug, Clone)]
pub struct Poe2ModifierStat {
    pub id: String,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone)]
pub struct Poe2ModifierDefinition {
    pub id: Poe2ModifierId,
    pub kind: Poe2ModifierKind,
    pub required_level: u32,
    pub stats: Vec<Poe2ModifierStat>,
    pub groups: Vec<String>,
    pub generation_type: GenerationType,
}

impl ModifierDefinitionIdentity for Poe2ModifierDefinition {
    type Id = Poe2ModifierId;
    fn modifier_definition_id(&self) -> Self::Id {
        self.id.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Poe2Effect {}

impl Game for Poe2 {
    type ItemBase = ItemClass;
    type ItemState = Poe2ItemState;
    type ModifierDefinitionId = Poe2ModifierId;
    type ModifierDefinition = Poe2ModifierDefinition;

    type ModifierInstance = Vec<i64>;

    type Effect = Poe2Effect;
    type EffectCondition = ();
    type EffectSourceId = String;
}

pub struct Poe2DefinitionRegistry {
    definitions: HashMap<Poe2ModifierId, Poe2ModifierDefinition>,
}

impl Poe2DefinitionRegistry {
    pub fn new(raw_mods: RawModsFile) -> Self {
        let mut definitions = HashMap::new();

        for (key, raw_mod) in raw_mods {
            let id = Poe2ModifierId(key);
            let def = Poe2ModifierDefinition {
                id: id.clone(),
                kind: Poe2ModifierKind(raw_mod.mod_type),
                required_level: raw_mod.required_level,
                stats: raw_mod
                    .stats
                    .into_iter()
                    .map(|s| Poe2ModifierStat {
                        id: s.id,
                        min: s.min,
                        max: s.max,
                    })
                    .collect(),
                groups: raw_mod.groups,
                generation_type: raw_mod.generation_type,
            };
            definitions.insert(id, def);
        }

        Self { definitions }
    }
}

impl ModifierDefinitionProvider<Poe2> for Poe2DefinitionRegistry {
    type Error = String;

    fn definition(&self, id: &Poe2ModifierId) -> Result<&Poe2ModifierDefinition, Self::Error> {
        self.definitions
            .get(id)
            .ok_or_else(|| format!("Modifier not found: {:?}", id.0))
    }
}

#[test]
fn test_create_poe2_item() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let base_items_path = format!("{}/data/base_items.json", manifest_dir);
    let base_items = read_json_file(&base_items_path).unwrap();

    let sinister_base = base_items
        .values()
        .find(|item| item.name == "Sinister Quarterstaff")
        .expect("Base item not found!");

    let mods_path = format!("{}/data/mods.json", manifest_dir);
    let raw_mods = parse_mods_json(&mods_path).unwrap();
    let registry = Poe2DefinitionRegistry::new(raw_mods);

    let item_state = Poe2ItemState {
        item_level: 83,
        quality: 20,
        rarity: ItemRarity::Rare,
        is_corrupted: false,
        base_name: sinister_base.name.clone(),
        drop_level: sinister_base.drop_level,
        properties: sinister_base.properties.clone(),
        requirements: sinister_base.requirements.clone(),
    };

    let item = ItemInstance::<Poe2, Unvalidated>::new(ItemClass::Warstaff, item_state);
    assert!(item.state().properties.attack_time.is_some());
    println!(
        "Created Item: {} (Level {})",
        item.state().base_name,
        item.state().item_level
    );
}
