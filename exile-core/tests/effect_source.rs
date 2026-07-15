mod support;

use exile_core::effect::EffectSource;

use support::*;

#[test]
fn passive_node_can_emit_multiple_effects() {
    let node = TestPassiveNode::ChaosInoculation;

    let entries = node.collect_effects();

    assert_eq!(entries.len(), 2);

    assert_eq!(entries[0].effect(), &TestEffect::ChaosImmune,);

    assert_eq!(
        entries[1].effect(),
        &TestEffect::SetMaximumLife { value: 1 },
    );
}

#[test]
fn passive_node_can_emit_no_effects() {
    let node = TestPassiveNode::Empty;

    let entries = node.collect_effects();

    assert!(entries.is_empty());
}

#[test]
fn passive_node_emits_conditional_effect() {
    let node = TestPassiveNode::FullLifeDamage;

    let entries = node.collect_effects();

    assert_eq!(entries.len(), 1);

    let entry = &entries[0];

    assert_eq!(entry.effect(), &TestEffect::IncreasedDamage { percent: 20 },);

    assert_eq!(
        entry.condition(),
        Some(&TestEffectCondition::EnemyOnFullLife,),
    );
}
