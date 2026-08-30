use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use serde::{Deserialize, Deserializer, Serialize};

use crate::ModType;

type Poe2Item = HashMap<String, Item>;

struct Class {
    category: String,
    category_id: String,
    name: String,
    influence_tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MinMax {
    min: u32,
    max: u32,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Properties {
    pub armour: Option<MinMax>,
    pub energy_shield: Option<MinMax>,
    pub evasion: Option<MinMax>,
    pub ward: Option<MinMax>,
    pub movement_speed: Option<i32>,
    pub block: Option<u32>,
    pub description: Option<String>,
    pub directions: Option<String>,
    pub stack_size: Option<u32>,
    pub stack_size_currency_tab: Option<u32>,
    pub full_stack_turns_into: Option<String>,
    pub charges_max: Option<u32>,
    pub charges_per_use: Option<u32>,
    pub duration: Option<u32>,
    pub life_per_use: Option<u32>,
    pub mana_per_use: Option<u32>,
    #[serde(deserialize_with = "deserialize_attack_time", default)]
    pub attack_time: Option<f32>,
    #[serde(deserialize_with = "deserialize_crit_chance", default)]
    pub critical_strike_chance: Option<f32>,
    pub physical_damage_max: Option<u32>,
    pub physical_damage_min: Option<u32>,
    pub range: Option<u32>,
    pub mana_burn_ms: Option<u32>,
    pub cooldown_ms: Option<u32>,
    pub monster_id: Option<String>,
    pub monster_ability_text: Option<String>,
    pub monster_category: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Requirements {
    pub strength: u32,
    pub dexterity: u32,
    pub intelligence: u32,
    pub level: u32,
}

#[derive(Debug, Deserialize)]
pub struct VisualIdentity {
    id: String,
    dds_file: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ItemDomain {
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
pub struct Item {
    pub drop_level: u16,
    pub debug: String,
    pub name: String,
    pub domain: ItemDomain,
    pub inherits_from: String,
    pub inventory_height: u8,
    pub inventory_width: u8,
    pub properties: Properties,
    pub requirements: Option<Requirements>,
    pub skills_granted: Option<Vec<String>>,
    pub tags: HashSet<String>,
    pub implicits: Vec<String>,
    pub visual_identity: VisualIdentity,
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
    pub required_level: u16,
    pub spawn_weights: Vec<TagWeight>,
    pub stats: Vec<StatRoll>,
    pub text: Option<String>,
    #[serde(rename = "type")]
    pub mod_type: ModType,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TagWeight {
    pub tag: String,
    pub weight: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HashedTagWeight {
    pub tag: u64,
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

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum GenerationType {
    Prefix,
    Suffix,
    Corrupted,
    Unique,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ItemClass {
    #[serde(rename = "Active Skill Gem")]
    ActiveSkillGem,
    #[serde(rename = "Amulet")]
    Amulet,
    #[serde(rename = "AtlasUpgradeItem")]
    AtlasUpgradeItem,
    #[serde(rename = "Belt")]
    Belt,
    #[serde(rename = "Body Armour")]
    BodyArmour,
    #[serde(rename = "Boots")]
    Boots,
    #[serde(rename = "Bow")]
    Bow,
    #[serde(rename = "Breachstone")]
    Breachstone,
    #[serde(rename = "BrequelFruit")]
    BrequelFruit,
    #[serde(rename = "Buckler")]
    Buckler,
    #[serde(rename = "Claw")]
    Claw,
    #[serde(rename = "ConventionTreasure")]
    ConventionTreasure,
    #[serde(rename = "Crossbow")]
    Crossbow,
    #[serde(rename = "Currency")]
    Currency,
    #[serde(rename = "Dagger")]
    Dagger,
    #[serde(rename = "DelveSocketableCurrency")]
    DelveSocketableCurrency,
    #[serde(rename = "DelveStackableSocketableCurrency")]
    DelveStackableSocketableCurrency,
    #[serde(rename = "DivinationCard")]
    DivinationCard,
    #[serde(rename = "Expedition2Logbooks")]
    Expedition2Logbooks,
    #[serde(rename = "ExpeditionLogbook")]
    ExpeditionLogbook,
    #[serde(rename = "FishingRod")]
    FishingRod,
    #[serde(rename = "Flail")]
    Flail,
    #[serde(rename = "Focus")]
    Focus,
    #[serde(rename = "GiftBox")]
    GiftBox,
    #[serde(rename = "Gloves")]
    Gloves,
    #[serde(rename = "HeistBlueprint")]
    HeistBlueprint,
    #[serde(rename = "HeistContract")]
    HeistContract,
    #[serde(rename = "HeistEquipmentReward")]
    HeistEquipmentReward,
    #[serde(rename = "HeistEquipmentTool")]
    HeistEquipmentTool,
    #[serde(rename = "HeistEquipmentUtility")]
    HeistEquipmentUtility,
    #[serde(rename = "HeistEquipmentWeapon")]
    HeistEquipmentWeapon,
    #[serde(rename = "Helmet")]
    Helmet,
    #[serde(rename = "IncubatorStackable")]
    IncubatorStackable,
    #[serde(rename = "IncursionArm")]
    IncursionArm,
    #[serde(rename = "IncursionLeg")]
    IncursionLeg,
    #[serde(rename = "InstanceLocalItem")]
    InstanceLocalItem,
    #[serde(rename = "ItemisedSanctum")]
    ItemisedSanctum,
    #[serde(rename = "Jewel")]
    Jewel,
    #[serde(rename = "LifeFlask")]
    LifeFlask,
    #[serde(rename = "ManaFlask")]
    ManaFlask,
    #[serde(rename = "Map")]
    Map,
    #[serde(rename = "MapFragment")]
    MapFragment,
    #[serde(rename = "MemoryLine")]
    MemoryLine,
    #[serde(rename = "Meta Skill Gem")]
    MetaSkillGem,
    #[serde(rename = "Omen")]
    Omen,
    #[serde(rename = "One Hand Axe")]
    OneHandAxe,
    #[serde(rename = "One Hand Mace")]
    OneHandMace,
    #[serde(rename = "One Hand Sword")]
    OneHandSword,
    #[serde(rename = "PinnacleKeyStackable")]
    PinnacleKeyStackable,
    #[serde(rename = "PinnacleKey_OLD")]
    PinnacleKeyOld,
    #[serde(rename = "QuestItem")]
    QuestItem,
    #[serde(rename = "Quiver")]
    Quiver,
    #[serde(rename = "Relic")]
    Relic,
    #[serde(rename = "Ring")]
    Ring,
    #[serde(rename = "SanctumSpecialRelic")]
    SanctumSpecialRelic,
    #[serde(rename = "Sceptre")]
    Sceptre,
    #[serde(rename = "SentinelDrone")]
    SentinelDrone,
    #[serde(rename = "Shield")]
    Shield,
    #[serde(rename = "SkillGemToken")]
    SkillGemToken,
    #[serde(rename = "SoulCore")]
    SoulCore,
    #[serde(rename = "Spear")]
    Spear,
    #[serde(rename = "StackableCurrency")]
    StackableCurrency,
    #[serde(rename = "Staff")]
    Staff,
    #[serde(rename = "Support Skill Gem")]
    SupportSkillGem,
    #[serde(rename = "Talisman")]
    Talisman,
    #[serde(rename = "TowerAugmentation")]
    TowerAugmentation,
    #[serde(rename = "TrapTool")]
    TrapTool,
    #[serde(rename = "Two Hand Axe")]
    TwoHandAxe,
    #[serde(rename = "Two Hand Mace")]
    TwoHandMace,
    #[serde(rename = "Two Hand Sword")]
    TwoHandSword,
    #[serde(rename = "UltimatumKey")]
    UltimatumKey,
    #[serde(rename = "UncutReservationGemStackable")]
    UncutReservationGemStackable,
    #[serde(rename = "UncutReservationGem_OLD")]
    UncutReservationGemOld,
    #[serde(rename = "UncutSkillGemStackable")]
    UncutSkillGemStackable,
    #[serde(rename = "UncutSkillGem_OLD")]
    UncutSkillGemOld,
    #[serde(rename = "UncutSupportGemStackable")]
    UncutSupportGemStackable,
    #[serde(rename = "UncutSupportGem_OLD")]
    UncutSupportGemOld,
    #[serde(rename = "UtilityFlask")]
    UtilityFlask,
    #[serde(rename = "VaultKey")]
    VaultKey,
    #[serde(rename = "Wand")]
    Wand,
    #[serde(rename = "Warstaff")]
    Warstaff,
    #[default]
    Unknown,
}

impl ItemClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemClass::ActiveSkillGem => "Active Skill Gem",
            ItemClass::Amulet => "Amulet",
            ItemClass::AtlasUpgradeItem => "AtlasUpgradeItem",
            ItemClass::Belt => "Belt",
            ItemClass::BodyArmour => "Body Armour",
            ItemClass::Boots => "Boots",
            ItemClass::Bow => "Bow",
            ItemClass::Breachstone => "Breachstone",
            ItemClass::BrequelFruit => "BrequelFruit",
            ItemClass::Buckler => "Buckler",
            ItemClass::Claw => "Claw",
            ItemClass::ConventionTreasure => "ConventionTreasure",
            ItemClass::Crossbow => "Crossbow",
            ItemClass::Currency => "Currency",
            ItemClass::Dagger => "Dagger",
            ItemClass::DelveSocketableCurrency => "DelveSocketableCurrency",
            ItemClass::DelveStackableSocketableCurrency => "DelveStackableSocketableCurrency",
            ItemClass::DivinationCard => "DivinationCard",
            ItemClass::Expedition2Logbooks => "Expedition2Logbooks",
            ItemClass::ExpeditionLogbook => "ExpeditionLogbook",
            ItemClass::FishingRod => "FishingRod",
            ItemClass::Flail => "Flail",
            ItemClass::Focus => "Focus",
            ItemClass::GiftBox => "GiftBox",
            ItemClass::Gloves => "Gloves",
            ItemClass::HeistBlueprint => "HeistBlueprint",
            ItemClass::HeistContract => "HeistContract",
            ItemClass::HeistEquipmentReward => "HeistEquipmentReward",
            ItemClass::HeistEquipmentTool => "HeistEquipmentTool",
            ItemClass::HeistEquipmentUtility => "HeistEquipmentUtility",
            ItemClass::HeistEquipmentWeapon => "HeistEquipmentWeapon",
            ItemClass::Helmet => "Helmet",
            ItemClass::IncubatorStackable => "IncubatorStackable",
            ItemClass::IncursionArm => "IncursionArm",
            ItemClass::IncursionLeg => "IncursionLeg",
            ItemClass::InstanceLocalItem => "InstanceLocalItem",
            ItemClass::ItemisedSanctum => "ItemisedSanctum",
            ItemClass::Jewel => "Jewel",
            ItemClass::LifeFlask => "LifeFlask",
            ItemClass::ManaFlask => "ManaFlask",
            ItemClass::Map => "Map",
            ItemClass::MapFragment => "MapFragment",
            ItemClass::MemoryLine => "MemoryLine",
            ItemClass::MetaSkillGem => "Meta Skill Gem",
            ItemClass::Omen => "Omen",
            ItemClass::OneHandAxe => "One Hand Axe",
            ItemClass::OneHandMace => "One Hand Mace",
            ItemClass::OneHandSword => "One Hand Sword",
            ItemClass::PinnacleKeyStackable => "PinnacleKeyStackable",
            ItemClass::PinnacleKeyOld => "PinnacleKey_OLD",
            ItemClass::QuestItem => "QuestItem",
            ItemClass::Quiver => "Quiver",
            ItemClass::Relic => "Relic",
            ItemClass::Ring => "Ring",
            ItemClass::SanctumSpecialRelic => "SanctumSpecialRelic",
            ItemClass::Sceptre => "Sceptre",
            ItemClass::SentinelDrone => "SentinelDrone",
            ItemClass::Shield => "Shield",
            ItemClass::SkillGemToken => "SkillGemToken",
            ItemClass::SoulCore => "SoulCore",
            ItemClass::Spear => "Spear",
            ItemClass::StackableCurrency => "StackableCurrency",
            ItemClass::Staff => "Staff",
            ItemClass::SupportSkillGem => "Support Skill Gem",
            ItemClass::Talisman => "Talisman",
            ItemClass::TowerAugmentation => "TowerAugmentation",
            ItemClass::TrapTool => "TrapTool",
            ItemClass::TwoHandAxe => "Two Hand Axe",
            ItemClass::TwoHandMace => "Two Hand Mace",
            ItemClass::TwoHandSword => "Two Hand Sword",
            ItemClass::UltimatumKey => "UltimatumKey",
            ItemClass::UncutReservationGemStackable => "UncutReservationGemStackable",
            ItemClass::UncutReservationGemOld => "UncutReservationGem_OLD",
            ItemClass::UncutSkillGemStackable => "UncutSkillGemStackable",
            ItemClass::UncutSkillGemOld => "UncutSkillGem_OLD",
            ItemClass::UncutSupportGemStackable => "UncutSupportGemStackable",
            ItemClass::UncutSupportGemOld => "UncutSupportGem_OLD",
            ItemClass::UtilityFlask => "UtilityFlask",
            ItemClass::VaultKey => "VaultKey",
            ItemClass::Wand => "Wand",
            ItemClass::Warstaff => "Warstaff",
            ItemClass::Unknown => "Unknown",
        }
    }
}

impl fmt::Display for ItemClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn read_json_file(path: &str) -> Result<Poe2Item, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let value: Poe2Item = serde_json::from_reader(reader)?;
    Ok(value)
}

pub fn parse_mods_json(file_path: &str) -> Result<RawModsFile, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let mods_map: RawModsFile = serde_json::from_reader(reader)?;

    Ok(mods_map)
}

#[test]
fn test_read_json_file() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file_path = format!("{}/data/base_items.json", manifest_dir);
    let item = read_json_file(&file_path).unwrap();
    assert!(!item.is_empty());
}
