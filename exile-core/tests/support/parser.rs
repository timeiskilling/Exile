use exile_core::item::{ItemInstance, ItemTextParser, ModifierTextParser, Unvalidated};

use super::{
    TestGame, TestItemBase, TestItemState, TestModifier, TestModifierKind, TestPassiveNodeId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestModifierTextPattern {
    Exact {
        text: &'static str,
    },

    Rolled {
        prefix: &'static str,
        suffix: &'static str,
    },

    Range {
        prefix: &'static str,
        separator: &'static str,
        suffix: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestModifierTextDefinition {
    pub definition_id: TestModifierKind,
    pub pattern: TestModifierTextPattern,
}

#[derive(Debug)]
pub struct TestModifierTextDefinitionProvider {
    definitions: Vec<TestModifierTextDefinition>,
}

impl TestModifierTextDefinitionProvider {
    pub fn new(definitions: Vec<TestModifierTextDefinition>) -> Self {
        Self { definitions }
    }

    pub fn definitions(&self) -> &[TestModifierTextDefinition] {
        &self.definitions
    }
}

impl Default for TestModifierTextDefinitionProvider {
    fn default() -> Self {
        Self::new(vec![
            TestModifierTextDefinition {
                definition_id: TestModifierKind::MaximumLife,

                pattern: TestModifierTextPattern::Rolled {
                    prefix: "+",
                    suffix: " to Maximum Life",
                },
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::MovementSpeed,

                pattern: TestModifierTextPattern::Rolled {
                    prefix: "",
                    suffix: "% increased Movement Speed",
                },
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::GrantsPassiveNode {
                    node_id: TestPassiveNodeId::ChaosInoculation,
                },

                pattern: TestModifierTextPattern::Exact {
                    text: "Grants Chaos Inoculation",
                },
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::GrantsPassiveNode {
                    node_id: TestPassiveNodeId::FullLifeDamage,
                },

                pattern: TestModifierTextPattern::Exact {
                    text: "Grants Full Life Damage",
                },
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::AddedPhysicalDamage,

                pattern: TestModifierTextPattern::Range {
                    prefix: "Adds ",
                    separator: " to ",
                    suffix: " Physical Damage",
                },
            },
        ])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestModifierTextParserError {
    InvalidNumber(String),
}

#[derive(Debug)]
pub struct TestModifierTextParser {
    definition_provider: TestModifierTextDefinitionProvider,
}

impl TestModifierTextParser {
    pub fn new(definition_provider: TestModifierTextDefinitionProvider) -> Self {
        Self {
            definition_provider,
        }
    }
}

impl Default for TestModifierTextParser {
    fn default() -> Self {
        Self::new(TestModifierTextDefinitionProvider::default())
    }
}

impl ModifierTextParser<TestGame> for TestModifierTextParser {
    type Error = TestModifierTextParserError;

    fn try_parse_modifier(
        &self,
        line: &str,
    ) -> Result<Option<(TestModifierKind, TestModifier)>, Self::Error> {
        for definition in self.definition_provider.definitions() {
            match definition.pattern {
                TestModifierTextPattern::Exact { text } => {
                    if line == text {
                        return Ok(Some((definition.definition_id, TestModifier::NoRoll)));
                    }
                }

                TestModifierTextPattern::Rolled { prefix, suffix } => {
                    let Some(value) = capture_value(line, prefix, suffix) else {
                        continue;
                    };

                    let roll = parse_modifier_number(value, line)?;

                    return Ok(Some((
                        definition.definition_id,
                        TestModifier::Rolled { roll },
                    )));
                }
                TestModifierTextPattern::Range {
                    prefix,
                    separator,
                    suffix,
                } => {
                    let Some((min_text, max_text)) =
                        capture_range_values(line, prefix, separator, suffix)
                    else {
                        continue;
                    };

                    let min = parse_modifier_number(min_text, line)?;
                    let max = parse_modifier_number(max_text, line)?;

                    return Ok(Some((
                        definition.definition_id,
                        TestModifier::Range { min, max },
                    )));
                }
            }
        }

        Ok(None)
    }
}

fn capture_value<'a>(line: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let without_prefix = line.strip_prefix(prefix)?;

    without_prefix.strip_suffix(suffix)
}

fn capture_range_values<'a>(
    line: &'a str,
    prefix: &str,
    separator: &str,
    suffix: &str,
) -> Option<(&'a str, &'a str)> {
    let without_prefix = line.strip_prefix(prefix)?;

    let without_suffix = without_prefix.strip_suffix(suffix)?;

    without_suffix.split_once(separator)
}

fn parse_modifier_number(
    value: &str,
    original_line: &str,
) -> Result<u16, TestModifierTextParserError> {
    value
        .parse::<u16>()
        .map_err(|_| TestModifierTextParserError::InvalidNumber(original_line.to_owned()))
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestItemTextParserError {
    MissingBase,
    MissingItemLevel,

    UnknownBase(String),

    InvalidNumber(String),

    Modifier(TestModifierTextParserError),

    UnknownLine(String),
}

#[derive(Debug)]
pub struct TestItemTextParser {
    modifier_parser: TestModifierTextParser,
}

impl TestItemTextParser {
    pub fn new(modifier_parser: TestModifierTextParser) -> Self {
        Self { modifier_parser }
    }
}

impl Default for TestItemTextParser {
    fn default() -> Self {
        Self::new(TestModifierTextParser::default())
    }
}

impl ItemTextParser<TestGame> for TestItemTextParser {
    type Error = TestItemTextParserError;

    fn parse(&self, text: &str) -> Result<ItemInstance<TestGame, Unvalidated>, Self::Error> {
        let mut base = None;
        let mut state = None;
        let mut modifiers = Vec::new();

        for raw_line in text.lines() {
            let line = raw_line.trim();

            if line.is_empty() {
                continue;
            }

            if let Some(base_name) = line.strip_prefix("Base: ") {
                base = Some(match base_name {
                    "Boots" => TestItemBase { is_boots: true },

                    "Other" => TestItemBase { is_boots: false },

                    unknown => {
                        return Err(TestItemTextParserError::UnknownBase(unknown.to_owned()));
                    }
                });

                continue;
            }

            if let Some(value) = line.strip_prefix("Item Level: ") {
                let item_level = value
                    .parse::<u16>()
                    .map_err(|_| TestItemTextParserError::InvalidNumber(line.to_owned()))?;

                state = Some(TestItemState { item_level });

                continue;
            }

            let parsed_modifier = self
                .modifier_parser
                .try_parse_modifier(line)
                .map_err(TestItemTextParserError::Modifier)?;

            if let Some(modifier) = parsed_modifier {
                modifiers.push(modifier);
                continue;
            }

            return Err(TestItemTextParserError::UnknownLine(line.to_owned()));
        }

        let base = base.ok_or(TestItemTextParserError::MissingBase)?;

        let state = state.ok_or(TestItemTextParserError::MissingItemLevel)?;

        Ok(ItemInstance::from_parts(base, state, modifiers))
    }
}
