use ahash::HashSet;
use exile_core::item::{ItemInstance, Unvalidated};
use exile_poe2::{
    item::{
        definition::Poe2DefinitionRegistry,
        state::{EquipSlot, ItemRarity, Poe2, Poe2ItemState, hash_string},
    },
    repoe_parse::{
        ItemClass::{self, Warstaff},
        parse_mods_json, read_json_file,
    },
};

#[test]
fn test_create_poe2_item() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let base_items_path = format!("{}/data/base_items.json", manifest_dir);
    let base_items = read_json_file(&base_items_path).unwrap();

    let sinister_base = base_items
        .values()
        .find(|item| item.name == "Sinister Quarterstaff")
        .expect("Base item not found!");

    let hash_tags: HashSet<u64> = sinister_base.tags.iter().map(|t| hash_string(t)).collect();
    let mods_path = format!("{}/data/mods.json", manifest_dir);
    let raw_mods = parse_mods_json(&mods_path).unwrap();
    let _registry = Poe2DefinitionRegistry::new(raw_mods);

    let item_state = Poe2ItemState {
        item_level: 83,
        quality: 20,
        rarity: ItemRarity::Rare,
        is_corrupted: false,
        base_name: sinister_base.name.clone(),
        drop_level: sinister_base.drop_level,
        properties: sinister_base.properties.clone(),
        requirements: sinister_base.requirements.clone(),
        tags: hash_tags,
        equip_slot: EquipSlot::MainHand,
    };

    let item = ItemInstance::<Poe2, Unvalidated>::new(ItemClass::Warstaff, item_state);
    assert!(item.state().properties.attack_time.is_some());
    println!(
        "Created Item: {} (Level {})",
        item.state().base_name,
        item.state().item_level
    );
}
