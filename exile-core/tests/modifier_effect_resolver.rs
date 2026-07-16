mod support;

use exile_core::effect::ModifierEffectResolver;

use support::*;

#[test]
fn resolves_movement_speed_modifier() {
    let resolver = TestModifierEffectResolver::default();

    let definition = create_definition();
    let modifier = TestModifier::Rolled { roll: 27 };

    let entries = resolver
        .resolve_modifier_effects(&definition, &modifier)
        .unwrap();

    assert_eq!(entries.len(), 1);

    assert_eq!(
        entries[0].effect(),
        &TestEffect::IncreasedMovementSpeed { percent: 27 },
    );
}

#[test]
fn unsupported_modifier_returns_error() {
    let resolver = TestModifierEffectResolver::default();

    let definition = TestModifierDefinition {
        kind: TestModifierKind::Unsupported,
        required_item_level: 80,
        min_roll: 0,
        max_roll: 0,
    };

    let modifier = TestModifier::Rolled { roll: 0 };

    let result = resolver.resolve_modifier_effects(&definition, &modifier);

    assert!(matches!(
        result,
        Err(TestEffectResolveError::UnsupportedModifier)
    ));
}

#[test]
fn propagates_missing_passive_node_error() {
    let passive_nodes = TestPassiveNodeProvider::new(Vec::new());

    let resolver = TestModifierEffectResolver::new(passive_nodes);

    let definition = grants_chaos_inoculation_definition();

    let result = resolver.resolve_modifier_effects(&definition, &TestModifier::NoRoll);

    assert!(matches!(
        result,
        Err(TestEffectResolveError::PassiveNodeProvider(
            TestPassiveNodeProviderError::NotFound(TestPassiveNodeId::ChaosInoculation)
        ))
    ));
}

#[test]
fn resolves_added_physical_damage_range() {
    let resolver = TestModifierEffectResolver::default();

    let definition = added_physical_damage_definition();

    let effects = resolver
        .resolve_modifier_effects(&definition, &TestModifier::Range { min: 10, max: 20 })
        .expect("physical damage modifier should resolve");

    assert_eq!(effects.len(), 1);

    let entry = &effects[0];

    assert_eq!(
        entry.effect(),
        &TestEffect::AddedPhysicalDamage { min: 10, max: 20 },
    );

    assert_eq!(entry.condition(), None);
}

#[test]
fn rejects_single_roll_for_added_physical_damage() {
    let resolver = TestModifierEffectResolver::default();

    let definition = added_physical_damage_definition();

    let result = resolver.resolve_modifier_effects(&definition, &TestModifier::Rolled { roll: 15 });

    assert!(matches!(
        result,
        Err(TestEffectResolveError::InvalidModifierPayload)
    ));
}
