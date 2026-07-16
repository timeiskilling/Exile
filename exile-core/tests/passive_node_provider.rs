mod support;

use exile_core::effect::PassiveNodeProvider;

use support::{
    TestPassiveNode, TestPassiveNodeId, TestPassiveNodeProvider, TestPassiveNodeProviderError,
};

#[test]
fn returns_passive_node_by_id() {
    let provider = TestPassiveNodeProvider::default();

    let node = provider
        .node(&TestPassiveNodeId::ChaosInoculation)
        .expect("Chaos Inoculation should exist");

    assert_eq!(node, &TestPassiveNode::ChaosInoculation,);
}

#[test]
fn returns_correct_node_when_multiple_exist() {
    let provider = TestPassiveNodeProvider::default();

    let node = provider
        .node(&TestPassiveNodeId::FullLifeDamage)
        .expect("Full Life Damage should exist");

    assert_eq!(node, &TestPassiveNode::FullLifeDamage,);
}

#[test]
fn returns_error_when_passive_node_is_missing() {
    let provider = TestPassiveNodeProvider::new(Vec::new());

    let result = provider.node(&TestPassiveNodeId::ChaosInoculation);

    assert!(matches!(
        result,
        Err(TestPassiveNodeProviderError::NotFound(
            TestPassiveNodeId::ChaosInoculation
        ))
    ));
}
