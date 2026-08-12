use std::collections::HashMap;

use crate::{
    item::state::{Poe2ModifierDefinition, Poe2ModifierId, Poe2ModifierKind, Poe2ModifierStat},
    repoe_parse::RawModsFile,
};

pub struct Poe2DefinitionRegistry {
    pub definitions: HashMap<Poe2ModifierId, Poe2ModifierDefinition>,
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
