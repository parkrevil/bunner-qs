use bunner_qs_rs::parsing::errors::ParseError;
use bunner_qs_rs::{ParseOptions, StringifyOptions};
use serde::Deserialize;
use serde_json::{Map as JsonMap, Value};

const QUERY_ALLOW_DATA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/query_allow.json"
));
const QUERY_REJECT_DATA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/query_reject.json"
));
const QUERY_ROUNDTRIP_DATA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/query_roundtrip.json"
));

pub fn allow_cases() -> Vec<SeedCase> {
    load_cases_from_str(QUERY_ALLOW_DATA)
}

pub fn reject_cases() -> Vec<SeedCase> {
    load_cases_from_str(QUERY_REJECT_DATA)
}

pub fn roundtrip_cases() -> Vec<RoundTripSeed> {
    load_roundtrip_cases_from_str(QUERY_ROUNDTRIP_DATA)
}

pub fn load_cases_from_str(data: &str) -> Vec<SeedCase> {
    serde_json::from_str(data).expect("seed JSON should parse")
}

pub fn load_roundtrip_cases_from_str(data: &str) -> Vec<RoundTripSeed> {
    serde_json::from_str(data).expect("roundtrip seed JSON should parse")
}

#[derive(Debug, Deserialize)]
pub struct SeedCase {
    pub name: String,
    pub input: String,
    pub expect: SeedExpect,
    #[serde(default)]
    pub options: Option<SeedOptions>,
}

#[derive(Debug, Deserialize)]
pub struct SeedOptions {
    #[serde(default)]
    pub space_as_plus: Option<bool>,
    #[serde(default)]
    pub max_params: Option<usize>,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct SeedStringifyOptions {
    #[serde(default)]
    pub space_as_plus: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RoundTripSeed {
    pub name: String,
    pub query: String,
    #[serde(default)]
    pub parse_options: Option<SeedOptions>,
    #[serde(default)]
    pub stringify_options: Option<SeedStringifyOptions>,
    #[serde(default)]
    pub normalized: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedExpect {
    Ok,
    DuplicateKey,
    InvalidPercentEncoding,
    InvalidCharacter,
    TooManyParameters,
    InputTooLong,
    DepthExceeded,
    UnmatchedBracket,
    UnexpectedQuestionMark,
    InvalidUtf8,
}

impl SeedCase {
    pub fn parse_options(&self) -> ParseOptions {
        build_parse_options(self.options.as_ref())
    }
}

impl RoundTripSeed {
    pub fn parse_options(&self) -> ParseOptions {
        build_parse_options(self.parse_options.as_ref())
    }

    pub fn stringify_options(&self) -> StringifyOptions {
        build_stringify_options(self.stringify_options.as_ref())
    }

    pub fn normalized_query(&self) -> Option<&str> {
        self.normalized.as_deref()
    }
}

pub fn assert_case_outcome(case: &SeedCase, result: Result<Value, ParseError>) {
    match case.expect {
        SeedExpect::Ok => {
            result.unwrap_or_else(|err| {
                panic!(
                    "case `{}` expected success but failed: {:?}",
                    case.name, err
                )
            });
        }
        SeedExpect::DuplicateKey => match result {
            Err(ParseError::DuplicateKey { .. }) => {}
            other => panic!(
                "case `{}` expected DuplicateKey, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InvalidPercentEncoding => match result {
            Err(ParseError::InvalidPercentEncoding { .. }) => {}
            other => panic!(
                "case `{}` expected InvalidPercentEncoding, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InvalidCharacter => match result {
            Err(ParseError::InvalidCharacter { .. }) => {}
            other => panic!(
                "case `{}` expected InvalidCharacter, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::TooManyParameters => match result {
            Err(ParseError::TooManyParameters { .. }) => {}
            other => panic!(
                "case `{}` expected TooManyParameters, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InputTooLong => match result {
            Err(ParseError::InputTooLong { .. }) => {}
            other => panic!(
                "case `{}` expected InputTooLong, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::DepthExceeded => match result {
            Err(ParseError::DepthExceeded { .. }) => {}
            other => panic!(
                "case `{}` expected DepthExceeded, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::UnmatchedBracket => match result {
            Err(ParseError::UnmatchedBracket { .. }) => {}
            other => panic!(
                "case `{}` expected UnmatchedBracket, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::UnexpectedQuestionMark => match result {
            Err(ParseError::UnexpectedQuestionMark { .. }) => {}
            other => panic!(
                "case `{}` expected UnexpectedQuestionMark, got {:?}",
                case.name, other
            ),
        },
        SeedExpect::InvalidUtf8 => match result {
            Err(ParseError::InvalidUtf8) => {}
            other => panic!("case `{}` expected InvalidUtf8, got {:?}", case.name, other),
        },
    }
}

pub fn normalize_empty(value: Value) -> Value {
    match value {
        Value::Null => Value::Object(JsonMap::new()),
        other => other,
    }
}

fn build_parse_options(config: Option<&SeedOptions>) -> ParseOptions {
    let mut opts = ParseOptions::default();
    if let Some(cfg) = config {
        if let Some(space) = cfg.space_as_plus {
            opts.space_as_plus = space;
        }
        if let Some(max_params) = cfg.max_params {
            opts.max_params = Some(max_params);
        }
        if let Some(max_length) = cfg.max_length {
            opts.max_length = Some(max_length);
        }
        if let Some(max_depth) = cfg.max_depth {
            opts.max_depth = Some(max_depth);
        }
    }
    opts
}

fn build_stringify_options(config: Option<&SeedStringifyOptions>) -> StringifyOptions {
    let mut opts = StringifyOptions::default();
    if let Some(SeedStringifyOptions {
        space_as_plus: Some(space),
    }) = config
    {
        opts.space_as_plus = *space;
    }
    opts
}
