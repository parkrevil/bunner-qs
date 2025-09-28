//! Public API surface for parsing and stringifying query strings.

pub use parse::{parse, parse_with};
pub use stringify::{stringify, stringify_with};

mod errors;
mod options;
mod parse;
mod stringify;

pub use errors::{
    ParseError, ParseResult, SerdeStringifyError, SerdeStringifyResult, StringifyError,
    StringifyResult,
};
pub use options::{ParseOptions, ParseOptionsBuilder, StringifyOptions, StringifyOptionsBuilder};
pub use options::{set_global_parse_diagnostics, set_global_serde_fastpath};
