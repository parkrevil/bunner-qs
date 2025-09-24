mod value;

pub use value::QsValue;

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Utf8,
    Iso88591,
}

impl Default for Charset {
    fn default() -> Self {
        Self::Utf8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateStrategy {
    Combine,
    First,
    Last,
}

impl Default for DuplicateStrategy {
    fn default() -> Self {
        Self::Combine
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitSetting {
    Finite(usize),
    Infinite,
}

impl LimitSetting {
    pub fn new(limit: usize) -> Self {
        Self::Finite(limit)
    }
}

impl Default for LimitSetting {
    fn default() -> Self {
        Self::Finite(usize::MAX)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepthSetting {
    Unlimited,
    Disabled,
    Finite(usize),
}

impl Default for DepthSetting {
    fn default() -> Self {
        Self::Finite(5)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delimiter {
    Str(String),
    Char(char),
    Regex(String),
    Other,
}

impl Default for Delimiter {
    fn default() -> Self {
        Self::Char('&')
    }
}

pub type DecodeFn = std::sync::Arc<
    dyn Fn(&str, &dyn Fn(&str, Charset, ValueKind) -> QsValue, Charset, ValueKind) -> QsValue
        + Send
        + Sync,
>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Key,
    Value,
}

#[derive(Clone)]
pub struct ParseOptions {
    pub allow_dots: bool,
    pub decode_dot_in_keys: Option<bool>,
    pub allow_empty_arrays: bool,
    pub strict_null_handling: bool,
    pub depth: DepthSetting,
    pub array_limit: isize,
    pub parameter_limit: LimitSetting,
    pub comma: bool,
    pub parse_arrays: bool,
    pub allow_prototypes: bool,
    pub allow_sparse: bool,
    pub ignore_query_prefix: bool,
    pub delimiter: Delimiter,
    pub charset_sentinel: bool,
    pub charset: Charset,
    pub interpret_numeric_entities: bool,
    pub decoder: Option<DecodeFn>,
    pub duplicates: DuplicateStrategy,
    pub throw_on_limit_exceeded: Option<bool>,
    pub plain_objects: bool,
    pub strict_depth: bool,
    pub allow_sparse_arrays: bool,
    pub comma_round_trip: bool,
    pub additional: HashMap<String, QsValue>,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            allow_dots: false,
            decode_dot_in_keys: None,
            allow_empty_arrays: false,
            strict_null_handling: false,
            depth: DepthSetting::default(),
            array_limit: 20,
            parameter_limit: LimitSetting::Finite(1000),
            comma: false,
            parse_arrays: true,
            allow_prototypes: false,
            allow_sparse: false,
            ignore_query_prefix: false,
            delimiter: Delimiter::default(),
            charset_sentinel: false,
            charset: Charset::default(),
            interpret_numeric_entities: false,
            decoder: None,
            duplicates: DuplicateStrategy::default(),
            throw_on_limit_exceeded: None,
            plain_objects: false,
            strict_depth: false,
            allow_sparse_arrays: false,
            comma_round_trip: false,
            additional: HashMap::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("parse() is not yet implemented")]
    Unimplemented,
    #[error("invalid option: {0}")]
    InvalidOption(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse_with_options<S: AsRef<str>>(
    _input: S,
    _options: ParseOptions,
) -> ParseResult<QsValue> {
    Err(ParseError::Unimplemented)
}

pub fn parse<S: AsRef<str>>(input: S) -> ParseResult<QsValue> {
    parse_with_options(input, ParseOptions::default())
}
