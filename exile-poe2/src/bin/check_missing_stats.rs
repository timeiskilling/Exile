use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

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
    let finalizer_path = Path::new("exile-poe2/src/calculation/finalizer.rs");
    let balancer_path = Path::new("exile-poe2/src/stat_balancer.rs");

    if !mods_path.exists() {
        eprintln!("Error: Cannot find data/mods.json. Run from project root.");
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
        let is_valid_gen_type = mod_data.generation_type == "prefix"
            || mod_data.generation_type == "suffix"
            /*|| mod_data.generation_type == "unique"*/;

        if is_valid_domain && is_valid_gen_type {
            for s in &mod_data.stats {
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

    println!("Total unique stat IDs in mods.json: {}", all_stats.len());

    // Read implemented stats
    let finalizer_src = fs::read_to_string(finalizer_path).unwrap_or_default();
    let balancer_src = fs::read_to_string(balancer_path).unwrap_or_default();

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
