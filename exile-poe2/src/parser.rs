use std::sync::Arc;

use ahash::HashSet;

use exile_core::item::{ItemInstance, ItemTextParser, Unvalidated};

use crate::{
    item::{
        definition::{ItemRegistry, Poe2DefinitionRegistry},
        state::{
            EquipSlot, ItemRarity, ModOrigin, Poe2, Poe2ItemState, Poe2ModifierDefinition,
            Poe2ModifierId, Poe2ModifierInstance, hash_string,
        },
    },
    repoe_parse::{Domain, Item, ItemClass, Properties, Requirements},
    stat_matcher::{ParsedStat, StatMatcher},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemHeader<'a> {
    pub item_class: Option<&'a str>,
    pub rarity: ItemRarity,
    pub name: Option<&'a str>,
    pub base: &'a str,
}

impl ItemHeader<'_> {
    pub fn needs_base_resolution(&self) -> bool {
        self.rarity == ItemRarity::Magic
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderError {
    MissingRarity,
    UnsupportedRarity(String),
    BadTitle { rarity: ItemRarity, lines: usize },
}

pub fn parse_header(text: &str) -> Result<ItemHeader<'_>, HeaderError> {
    let first = text.split("--------").next().unwrap_or("");

    let mut item_class = None;
    let mut rarity_raw = None;
    let mut title: Vec<&str> = Vec::with_capacity(2);

    for line in first.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if let Some(v) = line.strip_prefix("Item Class:") {
            item_class = Some(v.trim());
        } else if let Some(v) = line.strip_prefix("Rarity:") {
            rarity_raw = Some(v.trim());
        } else {
            title.push(line);
        }
    }

    let raw = rarity_raw.ok_or(HeaderError::MissingRarity)?;
    let rarity = ItemRarity::parse(raw);
    if rarity == ItemRarity::Unknown {
        return Err(HeaderError::UnsupportedRarity(raw.to_string()));
    }

    let (name, base) = match (rarity, title.as_slice()) {
        (ItemRarity::Rare | ItemRarity::Unique, [name, base]) => (Some(*name), *base),
        (_, [base]) => (None, *base),
        _ => {
            return Err(HeaderError::BadTitle {
                rarity,
                lines: title.len(),
            });
        }
    };

    Ok(ItemHeader {
        item_class,
        rarity,
        name,
        base,
    })
}

pub struct Poe2NinjaTextParser {
    items: Arc<ItemRegistry>,
    matcher: &'static StatMatcher,
    defs: Arc<Poe2DefinitionRegistry>,
}

impl Poe2NinjaTextParser {
    pub fn build_state(
        &self,
        item: &Item,
        h: &ItemHeader,
        item_level: u16,
        text: &str,
    ) -> Poe2ItemState {
        let (_, equip_slot) = determine_class_and_slot(h.base, &item.properties);

        Poe2ItemState {
            item_level,
            quality: parse_quality(text),
            rarity: h.rarity,
            is_corrupted: is_corrupted(text),
            base_name: h.base.to_string(),
            drop_level: item.drop_level,
            properties: item.properties.clone(),
            requirements: parse_text_requirements(text),
            tags: item.tags.iter().map(|tag| hash_string(tag)).collect(),
            equip_slot,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
}

fn origin_from_tag(tag: Option<&str>) -> ModOrigin {
    match tag {
        Some("rune") => ModOrigin::Rune,
        Some("fractured") => ModOrigin::Fractured,
        Some("desecrated") => ModOrigin::Desecrated,
        Some("crafted") => ModOrigin::Crafted,
        _ => ModOrigin::Dropped,
    }
}

fn strip_tag(line: &str) -> (String, Option<String>, bool) {
    let (mut s, mut tag, mut bonded) = (line.trim(), None, false);
    if s.ends_with(')')
        && let Some(i) = s.rfind(" (")
    {
        let t = &s[i + 2..s.len() - 1];
        if [
            "rune",
            "fractured",
            "desecrated",
            "crafted",
            "implicit",
            "enchant",
        ]
        .contains(&t)
        {
            tag = Some(t.to_string());
            s = &s[..i];
        }
    }
    if let Some(r) = s.strip_prefix("Bonded: ") {
        bonded = true;
        s = r;
    }
    (s.to_string(), tag, bonded)
}

fn parse_quality(text: &str) -> u16 {
    text.lines()
        .find_map(|line| {
            let value = line.trim().strip_prefix("Quality:")?;

            value
                .trim()
                .trim_start_matches('+')
                .split('%')
                .next()?
                .trim()
                .parse::<u16>()
                .ok()
        })
        .unwrap_or(0)
}

fn is_corrupted(text: &str) -> bool {
    text.lines().any(|line| line.trim() == "Corrupted")
}

fn parse_text_requirements(text: &str) -> Option<Requirements> {
    text.lines()
        .map(str::trim)
        .find(|line| line.starts_with("Requires:"))
        .map(parse_requirements)
}

impl ItemTextParser<Poe2> for Poe2NinjaTextParser {
    type Error = ParseError;

    fn parse(&self, text: &str) -> Result<ItemInstance<Poe2, Unvalidated>, Self::Error> {
        let err = |message: &str| ParseError::InvalidFormat(message.to_string());

        let h = parse_header(text).map_err(|e| ParseError::InvalidFormat(format!("{e:?}")))?;

        if h.needs_base_resolution() {
            return Err(err("magic items are not supported"));
        }

        let item = self
            .items
            .get_item(h.base)
            .ok_or_else(|| err("unknown base"))?;

        let item_level = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("Item Level:"))
            .and_then(|value| value.trim().parse::<u16>().ok())
            .ok_or_else(|| err("no item level"))?;

        let candidates = self.defs.mods_by_tags(item);

        let pool: HashSet<u64> = candidates
            .iter()
            .flat_map(|(definition, _)| definition.stats.iter().map(|stat| stat.raw_id_hash))
            .collect();

        let sections: Vec<&str> = text.split("--------").collect();

        let start = sections
            .iter()
            .position(|section| {
                section
                    .lines()
                    .any(|line| line.trim().starts_with("Item Level:"))
            })
            .ok_or_else(|| err("no Item Level section"))?
            + 1;

        let mut groups: Vec<(ModOrigin, Vec<ParsedStat>)> = Vec::new();

        for section in &sections[start..] {
            for raw in section
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
                let (line, tag, _bonded) = strip_tag(raw);
                let origin = origin_from_tag(tag.as_deref());

                let stats = self
                    .matcher
                    .parse_line(&line, |id| pool.contains(&hash_string(id)))
                    .or_else(|| {
                        let mut all = self.matcher.parse_all(&line);
                        if all.len() == 1 { all.pop() } else { None }
                    });

                let Some(stats) = stats else {
                    return Err(err(&format!("unrecognized modifier line: {raw}")));
                };

                match groups
                    .iter_mut()
                    .find(|(group_origin, _)| *group_origin == origin)
                {
                    Some((_, group_stats)) => group_stats.extend(stats),
                    None => groups.push((origin, stats)),
                }
            }
        }

        let pool_defs: Vec<&Poe2ModifierDefinition> = candidates
            .iter()
            .map(|(definition, _)| *definition)
            .collect();

        let mut modifiers = Vec::new();

        for (origin, stats) in &groups {
            let all_defs: Vec<&Poe2ModifierDefinition>;

            let defs: &[&Poe2ModifierDefinition] = if *origin == ModOrigin::Rune {
                all_defs = self.defs.definitions.values().collect();
                &all_defs
            } else {
                &pool_defs
            };

            let ResolveModsResult(resolved, left) = resolve_mods(defs, stats, *origin, item_level);

            if !left.is_empty() {
                let unresolved = left
                    .iter()
                    .map(|(hash, value)| format!("{:?} = {value}", self.defs.lookup_string(*hash)))
                    .collect::<Vec<_>>()
                    .join(", ");

                return Err(err(&format!(
                    "could not resolve modifier stats for origin {origin:?}: {unresolved}"
                )));
            }

            modifiers.extend(resolved);
        }

        let state = self.build_state(item, &h, item_level, text);

        Ok(ItemInstance::from_parts(item.item_class, state, modifiers))
    }
}

fn domain_ok(d: Domain, origin: ModOrigin) -> bool {
    match origin {
        ModOrigin::Desecrated => d == Domain::Desecrated,
        ModOrigin::Crafted => matches!(d, Domain::Item | Domain::Crafted),
        ModOrigin::Rune => true,
        _ => d == Domain::Item,
    }
}

fn match_def(def: &Poe2ModifierDefinition, left: &[(u64, i64)]) -> Option<Vec<usize>> {
    let mut picked: Vec<usize> = Vec::with_capacity(def.stats.len());
    for st in &def.stats {
        let (lo, hi) = (st.min.min(st.max), st.min.max(st.max));
        let i = (0..left.len()).find(|&i| {
            let (h, v) = left[i];
            h == st.raw_id_hash && (lo..=hi).contains(&v) && !picked.contains(&i)
        })?;
        picked.push(i);
    }
    (!picked.is_empty()).then_some(picked)
}

pub struct ResolveModsResult(Vec<(Poe2ModifierId, Poe2ModifierInstance)>, Vec<(u64, i64)>);

pub fn resolve_mods(
    defs: &[&Poe2ModifierDefinition],
    parsed: &[ParsedStat],
    origin: ModOrigin,
    item_level: u16,
) -> ResolveModsResult {
    let mut left: Vec<(u64, i64)> = parsed
        .iter()
        .map(|s| (hash_string(s.id), s.value))
        .collect();
    let mut out = Vec::new();

    while !left.is_empty() {
        let best = defs
            .iter()
            .filter(|d| {
                domain_ok(d.domain, origin)
                    && (origin == ModOrigin::Rune || d.required_level <= item_level)
            })
            .filter_map(|d| match_def(d, &left).map(|picked| (*d, picked)))
            .max_by_key(|(d, _)| (d.stats.len(), d.required_level));

        let Some((def, mut picked)) = best else { break };

        let rolls: Vec<i64> = picked.iter().map(|&i| left[i].1).collect();
        picked.sort_unstable_by(|a, b| b.cmp(a));
        for i in picked {
            left.remove(i);
        }
        out.push((def.id, Poe2ModifierInstance { rolls, origin }));
    }

    ResolveModsResult(out, left)
}

fn _extract_min_max(line: &str, prefix: &str) -> Option<(f32, f32)> {
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

fn _extract_value(line: &str, prefix: &str) -> Option<f32> {
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
