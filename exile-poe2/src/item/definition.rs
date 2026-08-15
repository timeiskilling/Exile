use std::collections::HashMap;

use crate::{
    item::state::{
        Poe2ModifierDefinition, Poe2ModifierId, Poe2ModifierKind, Poe2ModifierStat, hash_string,
    },
    repoe_parse::{HashedTagWeight, RawModsFile},
};

pub struct Poe2DefinitionRegistry {
    pub definitions: HashMap<Poe2ModifierId, Poe2ModifierDefinition>,
    pub string_dictionary: std::collections::HashMap<u64, String>,
}

impl Poe2DefinitionRegistry {
    pub fn new(raw_mods: RawModsFile) -> Self {
        let mut definitions = HashMap::new();
        let mut string_dictionary = std::collections::HashMap::new();

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
                    let stat_hash = hash_string(&s.id);
                    string_dictionary.insert(stat_hash, s.id.clone());
                    Poe2ModifierStat {
                        id_hash: stat_hash,
                        min: s.min,
                        max: s.max,
                        is_local: s.id.starts_with("local_"),
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
