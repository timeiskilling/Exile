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
                        "local_physical_damage_+%" => {
                            item.local_percent_physical += value as f64;
                        },
                        "local_life_leech_from_physical_damage_permyriad" => {
                            item.local_life_leech_from_physical_damage += (value as f64) / 100.0;
                        },
                        "local_mana_leech_from_physical_damage_permyriad" => {
                            item.local_mana_leech_from_physical_damage += (value as f64) / 100.0;
                        },
                        "local_life_gain_per_target" => {
                            item.local_life_gain_per_hit += value as u64;
                        },
                        "local_mana_gain_per_target" => {
                            item.local_mana_gain_per_hit += value as u64;
                        },
                        "local_attack_speed_+%" => {
                            item.local_attack_speed += value as f64;
                        },
                        "local_critical_strike_chance_+%" => {
                            item.local_critical_strike_chance += (value as f64) / 100.0;
                        },
                        "local_critical_strike_multiplier_+" => {
                            item.local_critical_strike_multiplier += value;
                        },
                        "local_block_chance_+%" => {
                            item.local_block_chance += value as u64;
                        },
                        "local_spirit_+%" => {
                            item.local_spirit_percent_increase += value as u64;
                        },
                        "number_of_additional_arrows" => {
                            item.number_of_additional_arrows += value as u64;
                        },
                        "chance_to_fire_1_additional_projectile_%_with_rollover_with_bow_attacks" => {
                            item.chance_to_fire_1_additional_projectile += value as u64;
                        },
                        "local_additional_charm_slots" => {
                            item.number_of_additional_charm_slots += value as u64;
                        },
                        "local_base_stun_duration_+%" => {
                            item.local_base_stun_duration += value as u64;
                        },
                        "local_hit_damage_stun_multiplier_+%" => {
                            item.local_hit_damage_stun_multiplier += value as u64;
                        },
                        "local_charges_added_+%" => {
                            item.local_flask_charges_gained += value as u64;
                        },
                        "local_max_charges_+%" => {
                            item.local_flask_max_charges += value as u64;
                        },
                        "local_charges_used_+%" => {
                            item.local_flask_charges_reduced_used += value;
                        },
                        "local_%_chance_to_gain_flask_charge_on_kill" => {
                            item.local_chance_to_gain_flask_charge_on_kill += value as u64;
                        },
                        "local_flask_gain_X_charges_every_minute" => {
                            item.local_flask_gain_x_charges_every_minute += value as u64;
                        },
                        "local_flask_recovery_speed_+%" => {
                            item.local_flask_recovery_speed_plus_percent += value as u64;
                        },
                        "local_flask_amount_to_recover_+%" => {
                            item.local_flask_amount_to_recover_plus_percent += value;
                        },
                        "local_flask_amount_to_recover_+%_when_on_low_life" => {
                            item.local_flask_amount_to_recover_plus_percent_when_on_low_life += value as u64;
                        },
                        "local_flask_amount_to_recover_+%_when_on_low_mana" => {
                            item.local_flask_amount_to_recover_plus_percent_when_on_low_mana += value as u64;
                        },
                        "local_flask_life_to_recover_+%" => {
                            item.local_flask_life_to_recover_plus_percent += value as u64;
                        },
                        "local_flask_mana_to_recover_+%" => {
                            item.local_flask_mana_to_recover_plus_percent += value as u64;
                        },
                        "local_flask_removes_%_of_life_recovery_from_mana_on_use" => {
                            item.local_flask_removes_of_life_recovery_from_mana_on_use += value as f64;
                        },
                        "local_flask_removes_%_of_mana_recovery_from_life_on_use" => {
                            item.local_flask_removes_of_mana_recovery_from_life_on_use += value as f64;
                        },
                        "local_flask_recovery_amount_%_to_recover_instantly" => {
                            item.local_flask_recover_instantly_percent += value as f64;
                        },
                        "local_flask_recovers_instantly" => {
                            item.local_flask_recover_instantly = true;
                        },
                        "local_flask_minion_heal_%" => {
                            item.local_flask_minion_heal_percent += value as f64;
                        },
                        "local_charm_duration_+%" => {
                            item.local_charm_duration_plus_percent += value as f64;
                        },
                        "local_maximum_prefixes_allowed_+" => {
                            item.local_maximum_prefixes_allowed += value;
                        },
                        "local_maximum_suffixes_allowed_+" => {
                            item.local_maximum_suffixes_allowed += value ;
                        },
                        "local_item_benefit_socketable_as_if_helmet" => {
                            item.local_item_benefit_socketable_as_if_helmet = true;
                        },
                        "local_item_additional_skill_slots" => {
                            item.local_item_additional_skill_slots += value as u64;
                        },
                        "local_maximum_quality_is_%" => {
                            item.local_maximum_quality_is_allowed += value as u64;
                        },
                        "local_item_benefit_socketable_as_if_gloves" => {
                            item.local_item_benefit_socketable_as_if_gloves = true;
                        },
                        "local_item_benefit_socketable_as_if_boots" => {
                            item.local_item_benefit_socketable_as_if_boots = true;
                        },
                        "local_charm_slots" => {
                            item.local_charm_slots += value as u64;
                        },
                        "local_flask_use_on_affected_by_freeze" => {
                            item.local_flask_use_on_affected_by_freeze = true;
                        },
                        "local_flask_use_on_affected_by_bleed" => {
                            item.local_flask_use_on_affected_by_bleed = true;
                        },
                        "local_flask_use_on_affected_by_poison" => {
                            item.local_flask_use_on_affected_by_poison = true;
                        },
                        "local_flask_use_on_affected_by_ignite" => {
                            item.local_flask_use_on_affected_by_ignite = true;
                        },
                        "local_flask_use_on_affected_by_shock" => {
                            item.local_flask_use_on_affected_by_shock = true;
                        },
                        "local_flask_use_on_stunned" => {
                            item.local_flask_use_on_stunned = true;
                        },
                        "local_flask_use_on_affected_by_slow" => {
                            item.local_flask_use_on_affected_by_slow = true;
                        },
                        "local_flask_use_on_fire_damage_taken" => {
                            item.local_flask_use_on_fire_damage_taken = true;
                        },
                        "local_flask_use_on_cold_damage_taken" => {
                            item.local_flask_use_on_cold_damage_taken = true;
                        },
                        "local_flask_use_on_lightning_damage_taken" => {
                            item.local_flask_use_on_lightning_damage_taken = true;
                        },
                        "local_flask_use_on_chaos_damage_taken" => {
                            item.local_flask_use_on_chaos_damage_taken = true;
                        },
                        "local_flask_use_on_killing_rare_or_unique_enemy" => {
                            item.local_flask_use_on_killing_rare_or_unique_enemy = true;
                        },
                        "local_charm_trigger_when_cursed" => {
                            item.local_charm_trigger_when_cursed = true;
                        },
                        "local_ward" => {
                            item.local_maximum_ward += value as u64;
                        },
                        "local_gain_X_rage_on_hit" => {
                            item.local_rage_on_hit += value as u64;
                        },
                        "local_weapon_accuracy_is_unaffected_by_distance" => {
                            item.local_weapon_accuracy_is_unaffected_by_distance = true;
                        },
                        "local_culling_strike" => {
                            item.local_culling_strike = true;
                        },
                        "local_chance_to_bleed_on_hit_%" => {
                            item.local_chance_to_bleed_on_hit += value as f64;
                        },
                        "local_cannot_be_thrown" => {
                            item.local_cannot_be_thrown = true;
                        },
                        "local_weapon_daze_chance_%" => {
                            item.local_weapon_daze_chance += value as f64;
                        },
                        "local_always_hit" => {
                            item.local_always_hit = true;
                        },
                        "local_explode_on_kill_with_crit_%_physical_damage_to_deal" => {
                            item.local_explode_on_kill_with_crit += value as f64;
                        },
                        "local_crush_on_hit" => {
                            item.local_crush_on_hit = true;
                        },
                        "local_weapon_implicit_hidden_%_base_damage_is_fire" => {
                            item.base_fire_damage += value as f64;
                        },
                        "local_display_grants_spear_throw_skill" => {
                            item.local_display_grants_spear_throw_skill = true;
                        },
                        "local_maim_on_hit_%" => {
                            item.local_maim_on_hit += value as f64;
                        },
                        "local_projectile_speed_+%" => {
                            item.local_projectile_speed += value as f64;
                        },
                        "local_+%_weapon_range" => {
                            item.local_weapon_range += value as f64;
                        },
                        "local_chance_to_blind_on_hit_%" => {
                            item.local_chance_to_blind_on_hit += value as f64;
                        },
                        "local_poison_on_hit_%" => {
                            item.local_poison_on_hit += value as f64;
                        },
                        "local_apply_X_armour_break_on_crit" => {
                            item.local_apply_x_armour_break_on_crit += value as u64;
                        },
                        "local_weapon_roll_crits_twice" => {
                            item.local_weapon_roll_crits_twice = true;
                        },
                        "local_attacks_cannot_be_blocked" => {
                            item.local_attacks_cannot_be_blocked = true;
                        },
                        "local_additional_attack_chain_chance_%" => {
                            item.local_chain_chance += value as f64;
                        },
                        "local_crossbow_no_ammo_skills_and_give_alternate_grenade_default_attack" => {
                            item.local_crossbow_no_ammo_skills_and_give_alternate_grenade_default_attack = true;
                        },
                    });
                }
            }
        }
    }
}
