use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

type Poe2Item = HashMap<String, Item>;

struct Class {
    category: String,
    category_id: String,
    name: String,
    influence_tags: Option<Vec<String>>,
}

struct ItemClass {
    class: Vec<Class>,
}
#[derive(Debug, Deserialize)]
struct MinMax {
    min: u32,
    max: u32,
}

#[derive(Debug, Deserialize)]
struct Properties {
    armour: Option<MinMax>,
    energy_shield: Option<MinMax>,
    evasion: Option<MinMax>,
    ward: Option<MinMax>,
    movement_speed: Option<i32>,
    block: Option<u32>,
    description: Option<String>,
    directions: Option<String>,
    stack_size: Option<u32>,
    stack_size_currency_tab: Option<u32>,
    full_stack_turns_into: Option<String>,
    charges_max: Option<u32>,
    charges_per_use: Option<u32>,
    duration: Option<u32>,
    life_per_use: Option<u32>,
    mana_per_use: Option<u32>,
    #[serde(deserialize_with = "deserialize_attack_time", default)]
    attack_time: Option<f32>,
    #[serde(deserialize_with = "deserialize_crit_chance", default)]
    critical_strike_chance: Option<f32>,
    physical_damage_max: Option<u32>,
    physical_damage_min: Option<u32>,
    range: Option<u32>,
    mana_burn_ms: Option<u32>,
    cooldown_ms: Option<u32>,
    monster_id: Option<String>,
    monster_ability_text: Option<String>,
    monster_category: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Requirements {
    strength: u32,
    dexterity: u32,
    intelligence: u32,
    level: u32,
}

#[derive(Debug, Deserialize)]
struct VisualIdentity {
    id: String,
    dds_file: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum ItemDomain {
    Undefined,
    Item,
    Flask,
    Misc,
    Area,
    Watchstone,
    HeistNpc,
    HeistArea,
    MapDevice,
    VaultKey,
    IncursionLimb,
    Tablet,
    SanctumRelic,
    SanctifiedRelic,
    MemoryLine,
    UltimatumKey,
    Sentinel,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct Item {
    drop_level: u32,
    Debug: String,
    name: String,
    domain: ItemDomain,
    inherits_from: String,
    inventory_height: u8,
    inventory_width: u8,
    properties: Properties,
    requirements: Option<Requirements>,
    skills_granted: Option<Vec<String>>,
    tags: Vec<String>,
    implicits: Vec<String>,
    visual_identity: VisualIdentity,
}

struct Items {
    items: Vec<Item>,
}

struct Tags {
    tags: Vec<String>,
}

fn deserialize_crit_chance<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<u32> = Option::deserialize(deserializer)?;
    Ok(opt.map(|val| val as f32 / 100.0))
}

fn deserialize_attack_time<'de, D>(deserializer: D) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<u32> = Option::deserialize(deserializer)?;
    Ok(opt.map(|val| if val == 0 { 0.0 } else { 1000.0 / (val as f32) }))
}

#[derive(Debug, Deserialize)]
pub struct RawMod {
    pub adds_tags: Vec<String>,
    pub domain: Domain,
    pub generation_type: GenerationType,
    pub generation_weights: Vec<TagWeight>,
    pub grants_effects: Vec<GrantedEffectRef>,
    pub groups: Vec<String>,
    pub implicit_tags: Vec<String>,
    pub is_essence_only: bool,
    pub name: String,
    pub required_level: u32,
    pub spawn_weights: Vec<TagWeight>,
    pub stats: Vec<StatRoll>,
    pub text: Option<String>,
    #[serde(rename = "type")]
    pub mod_type: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TagWeight {
    pub tag: String,
    pub weight: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StatRoll {
    pub id: String,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrantedEffectRef {
    pub granted_effect_id: String,
    pub level: u32,
}

pub type RawModsFile = HashMap<String, RawMod>;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GenerationType {
    Prefix,
    Suffix,
    Corrupted,
    Unique,
    Essence,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Item,
    Flask,
    Misc,
    Crafted,
    Monster,
    Area,
    HeistNpc,
    Desecrated,
    Chest,
    HeistArea,
    Strongbox,
    SanctumRelic,
    DelveArea,
    Atlas,
    Tablet,
    HeistTrinket,
    SynthesisA,
    MapDevice,
    SynthesisGlobals,
    ExpeditionRelic,
    Sentinel,
    SynthesisBonus,
    MemoryLine,
    UltimatumKey,
    Veiled,
    VaultKey,
    IncursionLimb,
    Leaguestone,
    Dummy,
    AfflictionJewel,
    #[serde(other)]
    Unknown,
}

fn read_json_file(path: &str) -> Result<Poe2Item, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let value: Poe2Item = serde_json::from_reader(reader)?;
    Ok(value)
}

#[test]
fn test_read_json_file() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_path = format!("{}/../base_items.json", manifest_dir);
    let item = read_json_file(&file_path).unwrap();
    assert!(!item.is_empty());
}
