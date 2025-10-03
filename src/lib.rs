pub mod prelude;

mod config;
mod memory;
mod model;
mod nested;
pub mod parsing;
mod serde_adapter;
mod stringify;
mod util;

pub use config::{
    DuplicateKeyBehavior, ParseOptions, ParseOptionsBuilder, StringifyOptions,
    StringifyOptionsBuilder,
};
pub use parsing::{ParseError, ParseResult, parse, parse_with};
pub use serde_adapter::SerdeQueryError;
pub use stringify::{
    SerdeStringifyError, SerdeStringifyResult, StringifyError, StringifyResult, stringify,
    stringify_with,
};

#[cfg(test)]
#[path = "../tests/common/parsing_helpers.rs"]
pub(crate) mod parsing_helpers;

#[cfg(test)]
#[path = "../tests/common/arena_helpers.rs"]
pub(crate) mod arena_helpers;
