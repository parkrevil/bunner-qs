use indexmap::IndexMap;

pub type QueryMap = IndexMap<String, Vec<String>>;

mod error;
mod options;
mod parse;
mod stringify;

pub use error::{ParseError, ParseResult, StringifyError, StringifyResult};
pub use options::{ParseOptions, StringifyOptions};
pub use parse::{parse, parse_with_options};
pub use stringify::{stringify, stringify_with_options};
