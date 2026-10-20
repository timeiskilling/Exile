use ahash::{HashSet, HashSetExt};

use exile_core::item::{ItemInstance, ItemTextParser, Unvalidated};

use crate::{
    item::state::{EquipSlot, ItemRarity, Poe2, Poe2ItemState},
    repoe_parse::{ItemClass, MinMax, Properties, Requirements},
};

pub struct Poe2NinjaTextParser;

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
}

impl ItemTextParser<Poe2> for Poe2NinjaTextParser {
    type Error = ParseError;

    fn parse(&self, text: &str) -> Result<ItemInstance<Poe2, Unvalidated>, Self::Error> {
        
    }
}

fn extract_min_max(line: &str, prefix: &str) -> Option<(f32, f32)> {
    let remainder = line.strip_prefix(prefix)?;
    let cleaned = remainder.split(' ').next()?; // remove " (augmented)"
    let parts: Vec<&str> = cleaned.split('-').collect();
    if parts.len() == 2 {
        let min = parts[0].parse::<f32>().ok()?;
        let max = parts[1].parse::<f32>().ok()?;
        Some((min, max))
    } else {
        None
    }
}

fn extract_value(line: &str, prefix: &str) -> Option<f32> {
    let remainder = line.strip_prefix(prefix)?;
    let cleaned = remainder.split('%').next()?.split(' ').next()?; // remove % and (augmented)
    cleaned.parse::<f32>().ok()
}

fn parse_requirements(line: &str) -> Requirements {
    let mut req = Requirements::default();
    let remainder = line.strip_prefix("Requires: ").unwrap_or(line);
    let parts: Vec<&str> = remainder.split(',').map(|s| s.trim()).collect();

    for part in parts {
        if let Some(lvl) = part.strip_prefix("Level ") {
            req.level = lvl.parse().unwrap_or(0);
        } else if let Some(dex) = part.strip_suffix(" Dex") {
            req.dexterity = dex.parse().unwrap_or(0);
        } else if let Some(str_val) = part.strip_suffix(" Str") {
            req.strength = str_val.parse().unwrap_or(0);
        } else if let Some(int_val) = part.strip_suffix(" Int") {
            req.intelligence = int_val.parse().unwrap_or(0);
        }
    }
    req
}

fn determine_class_and_slot(base_name: &str, props: &Properties) -> (ItemClass, EquipSlot) {
    let name_lower = base_name.to_lowercase();

    // Check Offhands
    if name_lower.contains("shield")
        || name_lower.contains("buckler")
        || name_lower.contains("targe")
    {
        return (ItemClass::Shield, EquipSlot::OffHand);
    }
    if name_lower.contains("quiver") {
        return (ItemClass::Quiver, EquipSlot::OffHand);
    }
    if name_lower.contains("focus") {
        return (ItemClass::Focus, EquipSlot::OffHand);
    }

    // Check Weapons
    if name_lower.contains("quarterstaff") || name_lower.contains("staff") {
        return (ItemClass::Warstaff, EquipSlot::MainHand);
    }
    if name_lower.contains("bow") || name_lower.contains("crossbow") {
        return (ItemClass::Bow, EquipSlot::MainHand);
    }
    if name_lower.contains("wand") {
        return (ItemClass::Wand, EquipSlot::MainHand);
    }
    if name_lower.contains("sword") {
        return (ItemClass::OneHandSword, EquipSlot::MainHand);
    }
    if name_lower.contains("mace") {
        return (ItemClass::OneHandMace, EquipSlot::MainHand);
    }
    if name_lower.contains("axe") {
        return (ItemClass::OneHandAxe, EquipSlot::MainHand);
    }

    // Accessories
    if name_lower.contains("ring") {
        return (ItemClass::Ring, EquipSlot::Ring1);
    }
    if name_lower.contains("amulet") {
        return (ItemClass::Amulet, EquipSlot::Amulet);
    }
    if name_lower.contains("belt") {
        return (ItemClass::Belt, EquipSlot::Belt);
    }

    // Armor
    if name_lower.contains("boots")
        || name_lower.contains("greaves")
        || name_lower.contains("shoes")
    {
        return (ItemClass::Boots, EquipSlot::Boots);
    }
    if name_lower.contains("gloves")
        || name_lower.contains("gauntlets")
        || name_lower.contains("mitts")
    {
        return (ItemClass::Gloves, EquipSlot::Gloves);
    }
    if name_lower.contains("helmet")
        || name_lower.contains("circlet")
        || name_lower.contains("mask")
        || name_lower.contains("crown")
    {
        return (ItemClass::Helmet, EquipSlot::Helmet);
    }
    if props.armour.is_some() || props.energy_shield.is_some() || props.evasion.is_some() {
        return (ItemClass::BodyArmour, EquipSlot::BodyArmour);
    }

    (ItemClass::Unknown, EquipSlot::None)
}
