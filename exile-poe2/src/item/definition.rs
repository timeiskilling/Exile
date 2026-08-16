use std::collections::{HashMap, HashSet};

use crate::{
    item::state::{
        Poe2ModifierDefinition, Poe2ModifierId, Poe2ModifierKind, Poe2ModifierStat,
        Poe2StatModifierKind, hash_string,
    },
    poe2_condition::parse_condition,
    poe2_scaling::parse_scaling,
    repoe_parse::{HashedTagWeight, RawModsFile},
};

fn find_valid_split(
    raw_id: &str,
    marker: &str,
    known_ids: &std::collections::HashSet<String>,
) -> Option<usize> {
    raw_id.match_indices(marker).find_map(|(idx, _)| {
        let prefix = &raw_id[..idx];
        (!prefix.is_empty() && known_ids.contains(prefix)).then_some(idx)
    })
}

fn extract_condition_or_scaling<'a>(
    raw_id: &'a str,
    known_ids: &'a std::collections::HashSet<String>,
) -> (&'a str, Poe2StatModifierKind) {
    const GATE_MARKERS: [&str; 4] = ["_when_", "_if_", "_while_", "_during_"];

    for marker in GATE_MARKERS {
        if let Some(pos) = find_valid_split(raw_id, marker, known_ids) {
            let clean = &raw_id[..pos];
            let cond = parse_condition(&raw_id[pos..]);
            return (clean, Poe2StatModifierKind::Conditional(cond));
        }
    }

    if let Some(pos) = find_valid_split(raw_id, "_per_", known_ids) {
        let clean = &raw_id[..pos];
        let scaling = parse_scaling(&raw_id[pos..]);
        return (clean, Poe2StatModifierKind::Scaled(scaling));
    }

    (raw_id, Poe2StatModifierKind::Plain)
}

pub struct Poe2DefinitionRegistry {
    pub definitions: HashMap<Poe2ModifierId, Poe2ModifierDefinition>,
    pub string_dictionary: std::collections::HashMap<u64, String>,
}

impl Poe2DefinitionRegistry {
    pub fn new(raw_mods: RawModsFile) -> Self {
        let known_stat_ids: HashSet<String> = raw_mods
            .values()
            .flat_map(|raw_mod| raw_mod.stats.iter().map(|s| s.id.clone()))
            .collect();

        let mut definitions = HashMap::new();
        let mut string_dictionary = HashMap::new();

        for (key, raw_mod) in raw_mods {
            let mod_id_hash = hash_string(&key);
            string_dictionary.insert(mod_id_hash, key.clone());
            let id = Poe2ModifierId(mod_id_hash);

            let kind_hash = hash_string(&raw_mod.mod_type);
            string_dictionary.insert(kind_hash, raw_mod.mod_type.clone());

            let stats = raw_mod
                .stats
                .into_iter()
                .map(|s| {
                    let (clean_id, kind) = extract_condition_or_scaling(&s.id, &known_stat_ids);

                    let stat_hash = hash_string(clean_id);
                    string_dictionary
                        .entry(stat_hash)
                        .or_insert_with(|| clean_id.to_string());

                    string_dictionary
                        .entry(hash_string(&s.id))
                        .or_insert_with(|| s.id.clone());

                    Poe2ModifierStat {
                        id_hash: stat_hash,
                        min: s.min,
                        max: s.max,
                        is_local: clean_id.starts_with("local_"),
                        kind,
                    }
                })
                .collect();

            let groups: Vec<u64> = raw_mod
                .groups
                .into_iter()
                .map(|g| {
                    let group_hash = hash_string(&g);
                    string_dictionary.insert(group_hash, g.clone());
                    group_hash
                })
                .collect();

            let spawn_weights: Vec<HashedTagWeight> = raw_mod
                .spawn_weights
                .into_iter()
                .map(|w| HashedTagWeight {
                    tag: hash_string(&w.tag),
                    weight: w.weight,
                })
                .collect();

            let def = Poe2ModifierDefinition {
                id,
                kind: Poe2ModifierKind(kind_hash),
                required_level: raw_mod.required_level,
                stats,
                groups,
                spawn_weights,
                generation_type: raw_mod.generation_type,
            };
            definitions.insert(id, def);
        }

        Self {
            definitions,
            string_dictionary,
        }
    }

    pub fn lookup_string(&self, hash: u64) -> Option<&String> {
        self.string_dictionary.get(&hash)
    }
}
