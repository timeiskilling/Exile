mod support;

use exile_core::item::ModifierTextParser;

use support::*;

#[test]
fn parses_maximum_life_modifier_from_definition() {
    let parser = TestModifierTextParser::default();

    let result = parser
        .try_parse_modifier("+25 to Maximum Life")
        .expect("maximum life modifier should parse");

    assert_eq!(
        result,
        Some((
            TestModifierKind::MaximumLife,
            TestModifier::Rolled { roll: 25 },
        )),
    );
}

#[test]
fn parses_movement_speed_modifier_from_definition() {
    let parser = TestModifierTextParser::default();

    let result = parser
        .try_parse_modifier("20% increased Movement Speed")
        .expect("movement speed modifier should parse");

    assert_eq!(
        result,
        Some((
            TestModifierKind::MovementSpeed,
            TestModifier::Rolled { roll: 20 },
        )),
    );
}

#[test]
fn parses_no_roll_modifier_from_exact_definition() {
    let parser = TestModifierTextParser::default();

    let result = parser
        .try_parse_modifier("Grants Chaos Inoculation")
        .expect("granted passive node should parse");

    assert_eq!(
        result,
        Some((
            TestModifierKind::GrantsPassiveNode {
                node_id: TestPassiveNodeId::ChaosInoculation,
            },
            TestModifier::NoRoll,
        )),
    );
}

#[test]
fn returns_none_when_no_text_definition_matches() {
    let parser = TestModifierTextParser::default();

    let result = parser
        .try_parse_modifier("Unknown modifier text")
        .expect("unknown text should not cause parser error");

    assert_eq!(result, None);
}

#[test]
fn returns_error_when_matched_roll_is_not_a_number() {
    let parser = TestModifierTextParser::default();

    let result = parser.try_parse_modifier("+wrong to Maximum Life");

    assert!(matches!(
        result,
        Err(
            TestModifierTextParserError::
                InvalidNumber(ref line)
        ) if line
            == "+wrong to Maximum Life"
    ));
}

#[test]
fn parser_uses_external_text_definitions() {
    let provider = TestModifierTextDefinitionProvider::new(vec![TestModifierTextDefinition {
        definition_id: TestModifierKind::MaximumLife,

        pattern: TestModifierTextPattern::Rolled {
            prefix: "Life +",
            suffix: "",
        },
    }]);

    let parser = TestModifierTextParser::new(provider);

    let result = parser
        .try_parse_modifier("Life +42")
        .expect("custom modifier definition should parse");

    assert_eq!(
        result,
        Some((
            TestModifierKind::MaximumLife,
            TestModifier::Rolled { roll: 42 },
        )),
    );
}

#[test]
fn parser_does_not_know_default_text_when_provider_omits_it() {
    let provider = TestModifierTextDefinitionProvider::new(Vec::new());

    let parser = TestModifierTextParser::new(provider);

    let result = parser
        .try_parse_modifier("+25 to Maximum Life")
        .expect("missing definition should return none");

    assert_eq!(result, None);
}

#[test]
fn parses_modifier_with_two_numeric_values() {
    let parser = TestModifierTextParser::default();

    let result = parser
        .try_parse_modifier("Adds 10 to 20 Physical Damage")
        .expect("physical damage modifier should parse");

    assert_eq!(
        result,
        Some((
            TestModifierKind::AddedPhysicalDamage,
            TestModifier::Range { min: 10, max: 20 },
        )),
    );
}

#[test]
fn returns_error_when_range_value_is_not_numeric() {
    let parser = TestModifierTextParser::default();

    let result = parser.try_parse_modifier("Adds ten to 20 Physical Damage");

    assert!(matches!(
        result,
        Err(
            TestModifierTextParserError::
                InvalidNumber(ref line)
        ) if line
            == "Adds ten to 20 Physical Damage"
    ));
}
