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
        let lines: Vec<&str> = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            return Err(ParseError::InvalidFormat("Empty input".to_string()));
        }

        let mut current_block = 0;
        let mut rarity = ItemRarity::Normal;

        let mut item_level = 1;
        let mut quality = 0;
        let mut is_corrupted = false;

        let mut properties = Properties::default();
        let mut requirements = Requirements::default();

        let mut blocks: Vec<Vec<&str>> = Vec::new();
        let mut current_lines = Vec::new();

        for line in lines {
            if line.starts_with("--------") {
                blocks.push(current_lines.clone());
                current_lines.clear();
            } else {
                current_lines.push(line);
            }
        }
        if !current_lines.is_empty() {
            blocks.push(current_lines);
        }

        if blocks.is_empty() {
            return Err(ParseError::InvalidFormat(
                "No valid blocks found".to_string(),
            ));
        }

        // Parse Block 0: Rarity and Name
        let header = &blocks[0];
        if header.is_empty() {
            return Err(ParseError::InvalidFormat("Missing header".to_string()));
        }

        if let Some(r_line) = header[0].strip_prefix("Rarity: ") {
            rarity = match r_line {
                "Normal" => ItemRarity::Normal,
                "Magic" => ItemRarity::Magic,
                "Rare" => ItemRarity::Rare,
                "Unique" => ItemRarity::Unique,
                _ => ItemRarity::Normal,
            };
        }

        // Base name is usually the last line of the header block
        let base_name: String = header.last().unwrap().to_string();

        // Parse properties blocks
        for block in &blocks[1..] {
            for line in block {
                if let Some(val) = line.strip_prefix("Item Level: ") {
                    item_level = val.parse().unwrap_or(1);
                } else if let Some(val) = line.strip_prefix("Quality: +") {
                    let cleaned = val.split('%').next().unwrap_or("0");
                    quality = cleaned.parse().unwrap_or(0);
                } else if line.starts_with("Physical Damage: ") {
                    if let Some(values) = extract_min_max(line, "Physical Damage: ") {
                        properties.physical_damage_min = Some(values.0 as u32);
                        properties.physical_damage_max = Some(values.1 as u32);
                    }
                } else if line.starts_with("Critical Hit Chance: ") {
                    if let Some(val) = extract_value(line, "Critical Hit Chance: ") {
                        properties.critical_strike_chance = Some(val);
                    }
                } else if line.starts_with("Attacks per Second: ") {
                    if let Some(val) = extract_value(line, "Attacks per Second: ") {
                        properties.attack_time = Some(1000.0 / val);
                    }
                } else if line.starts_with("Armour: ") {
                    if let Some(val) = extract_value(line, "Armour: ") {
                        properties.armour = Some(MinMax {
                            min: val as u32,
                            max: val as u32,
                        });
                    }
                } else if line.starts_with("Evasion Rating: ") {
                    if let Some(val) = extract_value(line, "Evasion Rating: ") {
                        properties.evasion = Some(MinMax {
                            min: val as u32,
                            max: val as u32,
                        });
                    }
                } else if line.starts_with("Energy Shield: ") {
                    if let Some(val) = extract_value(line, "Energy Shield: ") {
                        properties.energy_shield = Some(MinMax {
                            min: val as u32,
                            max: val as u32,
                        });
                    }
                } else if line.starts_with("Requires: ") {
                    requirements = parse_requirements(line);
                } else if *line == "Corrupted" {
                    is_corrupted = true;
                }
            }
        }

        let (item_class, equip_slot) = determine_class_and_slot(&base_name, &properties);

        let state = Poe2ItemState {
            item_level,
            quality,
            rarity,
            is_corrupted,
            base_name,
            drop_level: 1,
            properties,
            requirements: Some(requirements),
            tags: HashSet::new(),
            equip_slot,
        };

        // TODO: In the future, we would map the remaining blocks to Poe2ModifierInstances
        // using the definition registry.

        Ok(ItemInstance::new(item_class, state))
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
