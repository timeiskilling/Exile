mod support;

use exile_core::effect::modifier_effect_resolver::ModifierEffectResolver;

use support::*;

#[test]
fn resolves_movement_speed_modifier() {
    let resolver = TestModifierEffectResolver;

    let definition = create_definition();
    let modifier = TestModifier { roll: 27 };

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
    let resolver = TestModifierEffectResolver;

    let definition = TestModifierDefinition {
        kind: TestModifierKind::Unsupported,
        required_item_level: 80,
        min_roll: 0,
        max_roll: 0,
    };

    let modifier = TestModifier { roll: 0 };

    let result = resolver.resolve_modifier_effects(&definition, &modifier);

    assert!(matches!(
        result,
        Err(TestEffectResolveError::UnsupportedModifier)
    ));
}
