mod support;

use exile_core::effect::effect_collection::EffectCollection;

use support::*;

#[test]
fn collects_effects_from_multiple_sources() {
    let first_node = TestPassiveNode::ChaosInoculation;

    let second_node = TestPassiveNode::FullLifeDamage;

    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&first_node);
    collection.collect_from_source(&second_node);

    assert_eq!(collection.len(), 3);

    assert!(
        collection
            .iter()
            .any(|entry| { entry.effect() == &TestEffect::ChaosImmune })
    );

    assert!(
        collection
            .iter()
            .any(|entry| { entry.effect() == &TestEffect::SetMaximumLife { value: 1 } })
    );

    assert!(collection.iter().any(|entry| {
        entry.effect() == &TestEffect::IncreasedDamage { percent: 20 }
            && entry.condition() == Some(&TestEffectCondition::EnemyOnFullLife)
    }));
}

#[test]
fn effect_collection_can_be_consumed() {
    let mut collection = EffectCollection::<TestGame>::new();

    collection.collect_from_source(&TestPassiveNode::FullLifeDamage);

    let entries: Vec<_> = collection.into_iter().collect();

    assert_eq!(entries.len(), 1);
}
