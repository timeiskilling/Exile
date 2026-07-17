use exile_core::item::{ItemInstance, ItemTextParser, ModifierTextParser, Unvalidated};

use super::{
    TestGame, TestItemBase, TestItemState, TestModifier, TestModifierKind, TestPassiveNodeId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestModifierTextDecoder {
    NoRoll,
    Rolled,
    Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestModifierTextDefinition {
    pub definition_id: TestModifierKind,
    pub pattern: &'static str,
    pub decoder: TestModifierTextDecoder,
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
                pattern: "+{} to Maximum Life",
                decoder: TestModifierTextDecoder::Rolled,
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::MovementSpeed,
                pattern: "{}% increased Movement Speed",
                decoder: TestModifierTextDecoder::Rolled,
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::AddedPhysicalDamage,
                pattern: "Adds {} to {} Physical Damage",
                decoder: TestModifierTextDecoder::Range,
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::GrantsPassiveNode {
                    node_id: TestPassiveNodeId::ChaosInoculation,
                },
                pattern: "Grants Chaos Inoculation",
                decoder: TestModifierTextDecoder::NoRoll,
            },
            TestModifierTextDefinition {
                definition_id: TestModifierKind::GrantsPassiveNode {
                    node_id: TestPassiveNodeId::FullLifeDamage,
                },
                pattern: "Grants Full Life Damage",
                decoder: TestModifierTextDecoder::NoRoll,
            },
        ])
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestModifierTextParserError {
    InvalidNumber(String),

    InvalidDefinitionCaptureCount {
        definition_id: TestModifierKind,
        expected: usize,
        actual: usize,
    },

    AmbiguousMatch {
        line: String,
        definition_ids: Vec<TestModifierKind>,
    },
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
        let mut matches = self
            .definition_provider
            .definitions()
            .iter()
            .filter_map(|definition| {
                capture_values(definition.pattern, line).map(|captures| (definition, captures))
            })
            .collect::<Vec<_>>();

        if matches.is_empty() {
            return Ok(None);
        }

        if matches.len() > 1 {
            return Err(TestModifierTextParserError::AmbiguousMatch {
                line: line.to_owned(),
                definition_ids: matches
                    .iter()
                    .map(|(definition, _)| definition.definition_id)
                    .collect(),
            });
        }

        let Some((definition, captures)) = matches.pop() else {
            return Ok(None);
        };

        let modifier = decode_modifier(definition, &captures, line)?;

        Ok(Some((definition.definition_id, modifier)))
    }
}

fn capture_values<'a>(pattern: &str, line: &'a str) -> Option<Vec<&'a str>> {
    let parts = pattern.split("{}").collect::<Vec<_>>();
    let capture_count = parts.len().saturating_sub(1);

    if capture_count == 0 {
        return (pattern == line).then(Vec::new);
    }

    let mut remaining = line.strip_prefix(parts[0])?;
    let mut captures = Vec::with_capacity(capture_count);

    for index in 0..capture_count {
        let next_literal = parts[index + 1];
        let is_last_capture = index + 1 == capture_count;

        if is_last_capture {
            if next_literal.is_empty() {
                captures.push(remaining);
            } else {
                captures.push(remaining.strip_suffix(next_literal)?);
            }

            remaining = "";
            continue;
        }

        if next_literal.is_empty() {
            return None;
        }

        let literal_index = remaining.find(next_literal)?;
        let captured = &remaining[..literal_index];

        captures.push(captured);

        remaining = &remaining[literal_index + next_literal.len()..];
    }

    remaining.is_empty().then_some(captures)
}

fn decode_modifier(
    definition: &TestModifierTextDefinition,
    captures: &[&str],
    original_line: &str,
) -> Result<TestModifier, TestModifierTextParserError> {
    match definition.decoder {
        TestModifierTextDecoder::NoRoll => {
            validate_capture_count(definition, captures, 0)?;

            Ok(TestModifier::NoRoll)
        }

        TestModifierTextDecoder::Rolled => {
            validate_capture_count(definition, captures, 1)?;

            let roll = parse_modifier_number(captures[0], original_line)?;

            Ok(TestModifier::Rolled { roll })
        }

        TestModifierTextDecoder::Range => {
            validate_capture_count(definition, captures, 2)?;

            let min = parse_modifier_number(captures[0], original_line)?;
            let max = parse_modifier_number(captures[1], original_line)?;

            Ok(TestModifier::Range { min, max })
        }
    }
}

fn validate_capture_count(
    definition: &TestModifierTextDefinition,
    captures: &[&str],
    expected: usize,
) -> Result<(), TestModifierTextParserError> {
    let actual = captures.len();

    if actual != expected {
        return Err(TestModifierTextParserError::InvalidDefinitionCaptureCount {
            definition_id: definition.definition_id,
            expected,
            actual,
        });
    }

    Ok(())
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
