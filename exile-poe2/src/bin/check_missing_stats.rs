use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub type StatTransaltion = Vec<StatTransaltionElement>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatTransaltionElement {
    pub english: Vec<English>,
    #[serde(rename = "ids")]
    pub ids: Vec<String>,
    #[serde(rename = "trade_stats")]
    pub trade_stats: Option<Vec<TradeStat>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct English {
    pub condition: Vec<Condition>,
    pub format: Vec<Format>,
    pub index_handlers: Vec<Vec<IndexHandler>>,
    pub string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub negated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    #[serde(rename = "#")]
    Empty,
    #[serde(rename = "+#")]
    Format,
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IndexHandler {
    #[serde(rename = "add_one")]
    AddOne,
    #[serde(rename = "affliction_reward_type")]
    AfflictionRewardType,
    #[serde(rename = "canonical_stat")]
    CanonicalStat,
    #[serde(rename = "deciseconds_to_seconds")]
    DecisecondsToSeconds,
    #[serde(rename = "display_indexable_support")]
    DisplayIndexableSupport,
    #[serde(rename = "divide_by_fifteen_0dp")]
    DivideByFifteen0Dp,
    #[serde(rename = "divide_by_fifty")]
    DivideByFifty,
    #[serde(rename = "divide_by_five")]
    DivideByFive,
    #[serde(rename = "divide_by_four")]
    DivideByFour,
    #[serde(rename = "divide_by_one_hundred")]
    DivideByOneHundred,
    #[serde(rename = "divide_by_one_hundred_1dp")]
    DivideByOneHundred1Dp,
    #[serde(rename = "divide_by_one_hundred_2dp")]
    DivideByOneHundred2Dp,
    #[serde(rename = "divide_by_one_hundred_2dp_if_required")]
    DivideByOneHundred2DpIfRequired,
    #[serde(rename = "divide_by_one_hundred_and_negate")]
    DivideByOneHundredAndNegate,
    #[serde(rename = "divide_by_ten_0dp")]
    DivideByTen0Dp,
    #[serde(rename = "divide_by_ten_1dp")]
    DivideByTen1Dp,
    #[serde(rename = "divide_by_ten_1dp_if_required")]
    DivideByTen1DpIfRequired,
    #[serde(rename = "divide_by_three")]
    DivideByThree,
    #[serde(rename = "divide_by_twenty_then_double_0dp")]
    DivideByTwentyThenDouble0Dp,
    #[serde(rename = "divide_by_two_0dp")]
    DivideByTwo0Dp,
    Double,
    #[serde(rename = "mages_legacy_index")]
    MagesLegacyIndex,
    #[serde(rename = "milliseconds_to_seconds")]
    MillisecondsToSeconds,
    #[serde(rename = "milliseconds_to_seconds_0dp")]
    MillisecondsToSeconds0Dp,
    #[serde(rename = "milliseconds_to_seconds_1dp")]
    MillisecondsToSeconds1Dp,
    #[serde(rename = "milliseconds_to_seconds_2dp")]
    MillisecondsToSeconds2Dp,
    #[serde(rename = "milliseconds_to_seconds_2dp_if_required")]
    MillisecondsToSeconds2DpIfRequired,
    #[serde(rename = "mod_value_to_item_class")]
    ModValueToItemClass,
    Negate,
    #[serde(rename = "negate_and_double")]
    NegateAndDouble,
    #[serde(rename = "passive_hash")]
    PassiveHash,
    #[serde(rename = "passive_keystone_index")]
    PassiveKeystoneIndex,
    #[serde(rename = "per_minute_to_per_second")]
    PerMinuteToPerSecond,
    #[serde(rename = "per_minute_to_per_second_0dp")]
    PerMinuteToPerSecond0Dp,
    #[serde(rename = "per_minute_to_per_second_1dp")]
    PerMinuteToPerSecond1Dp,
    #[serde(rename = "per_minute_to_per_second_2dp")]
    PerMinuteToPerSecond2Dp,
    #[serde(rename = "per_minute_to_per_second_2dp_if_required")]
    PerMinuteToPerSecond2DpIfRequired,
    #[serde(rename = "plus_two_hundred")]
    PlusTwoHundred,
    #[serde(rename = "specific_skill")]
    SpecificSkill,
    #[serde(rename = "subtract_one")]
    SubtractOne,
    #[serde(rename = "times_twenty")]
    TimesTwenty,
    #[serde(rename = "tree_expansion_jewel_passive")]
    TreeExpansionJewelPassive,
    #[serde(rename = "ultimatum_wager_type_hash")]
    UltimatumWagerTypeHash,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeStat {
    pub id: String,
    pub text: String,
    #[serde(rename = "type")]
    pub trade_stat_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Augment,
    Crafted,
    Desecrated,
    Enchant,
    Explicit,
    Fractured,
    Implicit,
    Pseudo,
    Sanctum,
}

#[derive(serde::Deserialize)]
struct ModData {
    #[serde(default)]
    domain: String,
    #[serde(default)]
    generation_type: String,
    #[serde(default)]
    stats: Vec<StatEntry>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    r#type: Option<String>,
}

#[derive(serde::Deserialize)]
struct StatEntry {
    id: String,
}

struct StatInfo {
    sample_text: String,
    sample_mod: String,
    domain: String,
    mod_type: String,
}

fn main() {
    let mods_path = Path::new("data/mods.json");
    let stats_descriptions = Path::new("data/stat_descriptions.json");

    let finalizer_path = Path::new("exile-poe2/src/calculation/finalizer.rs");
    let balancer_path = Path::new("exile-poe2/src/stat_balancer.rs");

    if !mods_path.exists() {
        eprintln!("Error: Cannot find data/mods.json. Run from project root.");
        return;
    }

    if !stats_descriptions.exists() {
        eprintln!("Error: Cannot find data/stat_descriptions.json. Run from project root.");
        return;
    }

    println!("Reading mods.json...");
    let json_content = fs::read_to_string(mods_path).expect("Failed to read mods.json");
    let raw_mods: BTreeMap<String, ModData> =
        serde_json::from_str(&json_content).expect("Failed to parse mods.json");

    // Collect all unique stats with metadata (filtered: domain in [item, desecrated] && generation_type in [prefix, suffix])
    let mut all_stats: BTreeMap<String, StatInfo> = BTreeMap::new();
    for (mod_name, mod_data) in &raw_mods {
        let is_valid_domain = mod_data.domain == "item" || mod_data.domain == "desecrated";
        let is_valid_gen_type =
            mod_data.generation_type == "prefix" || mod_data.generation_type == "suffix";

        // Filter out legacy PoE1 maps, heist enchantments, etc.
        let mod_type_str = mod_data.r#type.as_deref().unwrap_or_default();
        let is_map = mod_name.starts_with("Map") || mod_type_str.starts_with("Map");
        let is_heist = mod_name.contains("Heist") || mod_type_str.contains("Heist");

        if is_valid_domain && is_valid_gen_type && !is_map && !is_heist {
            for s in &mod_data.stats {
                // Filter out legacy stat IDs
                let is_legacy = s.id.starts_with("map_")
                    || s.id.starts_with("heist_")
                    || s.id.contains("socketed_")
                    || s.id.contains("socketed_gems")
                    || s.id.contains("socketed_aura_gem")
                    || s.id.contains("socketed_support_gem");

                if !is_legacy {
                    all_stats.entry(s.id.clone()).or_insert_with(|| StatInfo {
                        sample_text: mod_data
                            .text
                            .clone()
                            .unwrap_or_default()
                            .replace('\n', " // "),
                        sample_mod: mod_name.clone(),
                        domain: mod_data.domain.clone(),
                        mod_type: mod_data.r#type.clone().unwrap_or_default(),
                    });
                }
            }
        }
    }

    println!("Total unique stat IDs in mods.json: {}", all_stats.len());

    // Read implemented stats
    let finalizer_src = fs::read_to_string(finalizer_path).unwrap_or_default();
    let balancer_src = fs::read_to_string(balancer_path).unwrap_or_default();

    let json_content = fs::read_to_string(stats_descriptions).expect("Failed to read mods.json");
    let raw_stat_transaltion: StatTransaltion = serde_json::from_str(&json_content)
        .expect("Failed to parse mods.json stats_descriptions.json");

    println!(
        "Total unique stat transaliton in stats_descriptions.json: {}",
        raw_stat_transaltion.len()
    );

    let mut handled_stats = BTreeSet::new();
    for stat_id in all_stats.keys() {
        let needle = format!("\"{}\"", stat_id);
        if finalizer_src.contains(&needle) || balancer_src.contains(&needle) {
            handled_stats.insert(stat_id.clone());
        }
    }

    println!("Handled stat IDs: {}", handled_stats.len());
    let missing_count = all_stats.len() - handled_stats.len();
    println!("Missing stat IDs: {}", missing_count);

    let in_code_exact = |id: &str| {
        let needle = format!("\"{id}\"");
        finalizer_src.contains(&needle) || balancer_src.contains(&needle)
    };

    let mut exact_ids: BTreeSet<String> = BTreeSet::new();
    let mut groups_exact = 0;

    for t in &raw_stat_transaltion {
        if t.ids.iter().any(|id| in_code_exact(id)) {
            groups_exact += 1;
        }
        for id in &t.ids {
            if in_code_exact(id) {
                exact_ids.insert(id.clone());
            }
        }
    }

    println!("translation groups (exact match): {groups_exact}");
    println!("translation ids (exact match):    {}", exact_ids.len());

    let extra: Vec<_> = exact_ids
        .iter()
        .filter(|id| !all_stats.contains_key(id.as_str()))
        .collect();
    println!(
        "handled in code, not in filtered mods.json: {}",
        extra.len()
    );

    let no_translation: Vec<_> = all_stats
        .keys()
        .filter(|id| !exact_ids.contains(id.as_str()))
        .collect();
    println!(
        "mods.json ids without translation: {}",
        no_translation.len()
    );
    for id in &no_translation {
        println!("  {id}");
    }

    // Split missing stats into Local vs Global
    let mut missing_local = BTreeMap::new();
    let mut missing_global = BTreeMap::new();

    for (stat_id, info) in &all_stats {
        if !handled_stats.contains(stat_id) {
            if stat_id.starts_with("local_") {
                missing_local.insert(stat_id, info);
            } else {
                missing_global.insert(stat_id, info);
            }
        }
    }

    // Generate Markdown report
    let mut md = String::new();
    md.push_str("# Missing Stats Report (`data/mods.json`)\n\n");
    md.push_str(&format!(
        "- **Total unique stat IDs:** {}\n- **Handled so far:** {}\n- **Remaining missing:** {} ({} local, {} global)\n\n",
        all_stats.len(),
        handled_stats.len(),
        missing_count,
        missing_local.len(),
        missing_global.len()
    ));

    md.push_str("## 1. Missing Local Stats (`local_...`)\n");
    md.push_str("> These should typically be added to `exile-poe2/src/stat_balancer.rs` in `match_stat_id!` or `Poe2ItemFinalStat`.\n\n");
    md.push_str("| Stat ID | Mod Type | Sample Text | Sample Mod |\n");
    md.push_str("| :--- | :--- | :--- | :--- |\n");
    for (id, info) in &missing_local {
        md.push_str(&format!(
            "| `{}` | `{}` | {} | `{}` |\n",
            id, info.mod_type, info.sample_text, info.sample_mod
        ));
    }

    md.push_str("\n## 2. Missing Global Stats\n");
    md.push_str("> These should typically be added to `exile-poe2/src/calculation/finalizer.rs` in `stat_buckets!`.\n\n");
    md.push_str("| Stat ID | Mod Type | Sample Text | Sample Mod |\n");
    md.push_str("| :--- | :--- | :--- | :--- |\n");
    for (id, info) in &missing_global {
        md.push_str(&format!(
            "| `{}` | `{}` | {} | `{}` |\n",
            id, info.mod_type, info.sample_text, info.sample_mod
        ));
    }

    let out_file = "missing_stats.md";
    fs::write(out_file, md).expect("Failed to write missing_stats.md");
    println!("Report generated successfully at: {}", out_file);
}
