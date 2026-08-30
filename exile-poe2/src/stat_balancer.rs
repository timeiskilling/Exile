use crate::{ModType, calculation::finalizer::Poe2FinalStat, item::state::EquipSlot};

macro_rules! match_stat_id {
    ($target_id:expr, { $( $stat:literal => $action:block ),* $(,)? }) => {
        $(
            if $target_id == crate::item::state::hash_string($stat) {
                $action
            } else
        )*
        {
            // fallback (do nothing if it matches no branches)
        }
    };
}

impl ModType {
    pub fn resolve_modifier(
        &self,
        stat_id: u64,
        value: i64,
        slot: Option<EquipSlot>,
        final_stat: &mut Poe2FinalStat,
    ) {
        match self {
            ModType::AbyssTargetMod => {}
            // Because multiple ModTypes can grant the exact same underlying stat_ids (like local_base_physical_damage_reduction_rating),
            // you don't actually need to match on the ModType for simple local flat/percent additions!
            // We can handle all of these dynamically by just looking at the stat_id!
            _ => {
                if let Some(slot) = slot {
                    let item = final_stat.equipment.entry(slot).or_default();

                    match_stat_id!(stat_id, {
                        "local_base_physical_damage_reduction_rating" => {
                            item.local_flat_armour += value as u64;
                        },
                        "local_physical_damage_reduction_rating_+%" => {
                            item.local_percent_armour += (value as f64) / 100.0;
                        },
                        "local_energy_shield" => {
                            item.local_energy_shield += value as u64;
                        },
                        "local_energy_shield_+%" => {
                            item.local_percent_energy_shield += (value as f64) / 100.0;
                        },
                        "local_base_evasion_rating" => {
                            item.local_flat_evasion += value as u64;
                        },
                        "local_evasion_rating_+%" => {
                            item.local_percent_evasion += (value as f64) / 100.0;
                        },
                        "local_armour_and_evasion_+%" => {
                            item.local_percent_armour += (value as f64) / 100.0;
                            item.local_percent_evasion += (value as f64) / 100.0;
                        },
                        "local_armour_and_energy_shield_+%" => {
                            item.local_percent_armour += (value as f64) / 100.0;
                            item.local_percent_energy_shield += (value as f64) / 100.0;
                        },
                        "local_evasion_and_energy_shield_+%" => {
                            item.local_percent_evasion += (value as f64) / 100.0;
                            item.local_percent_energy_shield += (value as f64) / 100.0;
                        },
                        "local_attribute_requirements_+%" => {
                            let multiplier = 1.0 + (value as f32 / 100.0);

                            item.requirements.strength = (item.requirements.strength as f32 * multiplier) as u32;
                            item.requirements.dexterity = (item.requirements.dexterity as f32 * multiplier) as u32;
                            item.requirements.intelligence = (item.requirements.intelligence as f32 * multiplier) as u32;
                        },
                        "local_armour_and_evasion_and_energy_shield_+%" => {
                            item.local_percent_armour += (value as f64) / 100.0;
                            item.local_percent_evasion += (value as f64) / 100.0;
                            item.local_percent_energy_shield += (value as f64) / 100.0;
                        },
                        "local_minimum_added_physical_damage" => {
                            item.local_flat_physical_min += value as u64;
                        },
                        "local_maximum_added_physical_damage" => {
                            item.local_flat_physical_max += value as u64;
                        },
                        "local_minimum_added_fire_damage" => {
                            item.local_flat_fire_min += value as u64;
                        },
                        "local_maximum_added_fire_damage" => {
                            item.local_flat_fire_max += value as u64;
                        },
                        "local_minimum_added_cold_damage" => {
                            item.local_flat_cold_min += value as u64;
                        },
                        "local_maximum_added_cold_damage" => {
                            item.local_flat_cold_max += value as u64;
                        },
                        "local_minimum_added_lightning_damage" => {
                            item.local_flat_lightning_min += value as u64;
                        },
                        "local_maximum_added_lightning_damage" => {
                            item.local_flat_lightning_max += value as u64;
                        },
                    });
                }
            }
        }
    }
}
