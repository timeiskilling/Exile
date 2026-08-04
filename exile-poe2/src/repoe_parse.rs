use serde::{Deserialize, Deserializer};

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
struct Properties {
    armour: Option<u32>,
    energy_shield: Option<u32>,
    evasion: Option<u32>,
    ward: Option<u32>,
    movement_speed: Option<u32>,
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

struct Requirements {
    strength: u32,
    dexterity: u32,
    intelligence: u32,
    level: u32,
}

struct Item {
    drop_level: u32,
    item_class: String,
    name: String,
    properties: Properties,
    requirements: Option<Requirements>,
    skills_granted: Option<Vec<String>>,
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
