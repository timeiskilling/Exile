mod support;

use exile_core::item::{ItemTextParser, ItemValidator};
use support::*;

#[test]
fn parses_item_text_into_unvalidated_item() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Base: Boots
        Item Level: 86
        +25 to Maximum Life
        20% increased Movement Speed
        Grants Chaos Inoculation
    "#;

    let item = parser.parse(text).expect("item text should be parsed");

    assert!(item.base().is_boots);
    assert_eq!(item.state().item_level, 86);

    assert_eq!(item.revision(), 0);
    assert_eq!(item.modifiers().len(), 3);

    let maximum_life = &item.modifiers()[0];

    assert_eq!(maximum_life.definition_id(), &TestModifierKind::MaximumLife,);

    assert_eq!(maximum_life.modifier(), &TestModifier::Rolled { roll: 25 },);

    let movement_speed = &item.modifiers()[1];

    assert_eq!(
        movement_speed.definition_id(),
        &TestModifierKind::MovementSpeed,
    );

    assert_eq!(
        movement_speed.modifier(),
        &TestModifier::Rolled { roll: 20 },
    );

    let granted_node = &item.modifiers()[2];

    assert_eq!(
        granted_node.definition_id(),
        &TestModifierKind::GrantsPassiveNode {
            node_id: TestPassiveNodeId::ChaosInoculation,
        },
    );

    assert_eq!(granted_node.modifier(), &TestModifier::NoRoll,);
}

#[test]
fn parsed_item_can_be_validated() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Base: Boots
        Item Level: 86
        +25 to Maximum Life
        20% increased Movement Speed
        Grants Chaos Inoculation
    "#;

    let draft = parser.parse(text).expect("item text should be parsed");

    let provider = TestModifierDefinitionProvider::new(vec![
        maximum_life_definition(),
        movement_speed_definition(),
        grants_chaos_inoculation_definition(),
    ]);

    let validator = TestItemValidator::new(&provider);

    let item = draft
        .validate(&validator)
        .expect("parsed item should be valid");

    assert_eq!(item.modifiers().len(), 3);
    assert_eq!(item.revision(), 0);
}

#[test]
fn syntactically_valid_item_can_fail_domain_validation() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Base: Boots
        Item Level: 10
        20% increased Movement Speed
    "#;

    let draft = parser
        .parse(text)
        .expect("syntax should be parsed successfully");

    let provider = TestModifierDefinitionProvider::new(vec![movement_speed_definition()]);

    let validator = TestItemValidator::new(&provider);

    let result = validator.validate_item(&draft);

    assert!(matches!(
        result,
        Err(TestItemValidationError::ItemLevelTooLow(
            TestModifierKind::MovementSpeed
        ))
    ));
}

#[test]
fn returns_error_for_unknown_modifier_line() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Base: Boots
        Item Level: 86
        50% increased Unknown Power
    "#;

    let result = parser.parse(text);

    assert!(matches!(
        result,
        Err(TestItemTextParserError::UnknownLine(_))
    ));
}

#[test]
fn returns_error_when_base_is_missing() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Item Level: 86
        +25 to Maximum Life
    "#;

    let result = parser.parse(text);

    assert!(matches!(result, Err(TestItemTextParserError::MissingBase)));
}

#[test]
fn returns_error_when_item_level_is_missing() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Base: Boots
        +25 to Maximum Life
    "#;

    let result = parser.parse(text);

    assert!(matches!(
        result,
        Err(TestItemTextParserError::MissingItemLevel)
    ));
}

#[test]
fn returns_error_for_invalid_numeric_value() {
    let parser = TestItemTextParser::default();

    let text = r#"
        Base: Boots
        Item Level: eighty-six
        +25 to Maximum Life
    "#;

    let result = parser.parse(text);

    assert!(matches!(
        result,
        Err(TestItemTextParserError::InvalidNumber(_))
    ));
}
