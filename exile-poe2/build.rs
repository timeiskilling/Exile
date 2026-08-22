use std::fmt::Write;
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    path::PathBuf,
};

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let mods_path = std::path::Path::new(manifest_dir)
        .join("..")
        .join("data")
        .join("mods.json");

    println!("cargo:rerun-if-changed={}", mods_path.display());

    let json = fs::read_to_string(&mods_path).expect("Failed to read mods.json");

    let mods: BTreeMap<String, Mod> =
        serde_json::from_str(&json).expect("Failed to parse mods.json");

    let types: BTreeSet<String> = mods
        .values()
        .filter(|m| {
            (m.domain == "item" || m.domain == "desecrated")
                && (m.generation_type == "prefix" || m.generation_type == "suffix")
        })
        .map(|m| m.r#type.clone())
        .collect();

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));

    let output = out_dir.join("mod_type.rs");

    let mut generated = String::new();

    generated.push_str("// AUTO-GENERATED. DO NOT EDIT.\n\n");

    generated.push_str("#[allow(non_camel_case_types)]\n");
    generated.push_str("#[allow(clippy::all)]\n");

    generated.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    generated.push_str("pub enum ModType {\n");

    for ty in &types {
        let variant = sanitize_ident(ty);
        writeln!(generated, "    {},", variant).expect("Failed to write string");
    }

    generated.push_str("}\n\n");

    // &str -> ModType
    generated.push_str("impl std::str::FromStr for ModType {\n");
    generated.push_str("    type Err = ();\n\n");
    generated.push_str("    fn from_str(s: &str) -> Result<Self, Self::Err> {\n");
    generated.push_str("        match s {\n");

    for ty in &types {
        let variant = sanitize_ident(ty);
        writeln!(generated, "            {:?} => Ok(Self::{}),", ty, variant)
            .expect("Failed to write string");
    }

    generated.push_str("            _ => Err(()),\n");
    generated.push_str("        }\n");
    generated.push_str("    }\n");
    generated.push_str("}\n");

    fs::write(&output, generated).expect("Failed to write mod_type.rs");
}

#[derive(serde::Deserialize)]
struct Mod {
    #[serde(rename = "type")]
    r#type: String,
    domain: String,
    generation_type: String,
}

fn sanitize_ident(name: &str) -> String {
    let mut sanitized = name
        .replace('%', "Percent")
        .replace('+', "Plus")
        .replace('-', "Minus");

    sanitized.retain(|c| c.is_ascii_alphanumeric() || c == '_');

    if sanitized.starts_with(|c: char| c.is_ascii_digit()) {
        sanitized = format!("_{}", sanitized);
    }

    sanitized
}
