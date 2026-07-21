mod support;

use exile_core::{
    effect::{EffectCollection, ItemEffectCollector},
    item::ItemInstance,
};

use support::{
    TestGame, TestItemValidator, TestModifier, TestModifierDefinitionProvider,
    TestModifierEffectResolver, TestModifierKind, movement_speed_definition,
};

use crate::support::{TestEffect, TestItemBase, TestItemState};

#[test]
fn collects_effects_from_multiple_items() {
    let definitions = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let validator = TestItemValidator::new(&definitions);

    let items = [20, 25, 30].map(|roll| {
        ItemInstance::<TestGame>::from_parts(
            TestItemBase { is_boots: true },
            TestItemState { item_level: 86 },
            vec![(
                TestModifierKind::MovementSpeed,
                TestModifier::Rolled { roll },
            )],
        )
        .validate(&validator)
        .expect("test item should be valid")
    });

    let resolver = TestModifierEffectResolver::default();

    let collector = ItemEffectCollector::new(&definitions, &resolver);

    let mut collection = EffectCollection::<TestGame>::new();

    collection
        .collect_from_items(&collector, items.iter())
        .expect("item effect collection should succeed");

    assert_eq!(collection.len(), 3);

    let percents: Vec<_> = collection
        .iter()
        .filter_map(|entry| match entry.effect() {
            TestEffect::IncreasedMovementSpeed { percent } => Some(*percent),

            _ => None,
        })
        .collect();

    assert_eq!(percents, vec![20, 25, 30],);
}
