use indexmap::IndexMap;

pub type QueryMap = IndexMap<String, Vec<String>>;

mod error;
mod options;
mod parse;
mod stringify;
mod value;

pub use error::{ParseError, ParseResult, StringifyError, StringifyResult};
pub use options::{ParseOptions, ParseOptionsBuilder, StringifyOptions, StringifyOptionsBuilder};
pub use parse::{parse, parse_with_options};
pub use stringify::{Sorter, stringify, stringify_with_options, stringify_with_sorter};
pub use value::{SingleValueError, SingleValueResult, from_single_map, to_single_map};
