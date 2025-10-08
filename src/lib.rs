pub mod prelude;

mod config;
mod memory;
mod model;
mod nested;
pub mod parsing;
mod qs;
mod serde_adapter;
pub mod stringify;
mod util;

pub use config::{DuplicateKeyBehavior, OptionsValidationError, ParseOptions, StringifyOptions};
pub use qs::{Qs, QsParseError, QsStringifyError};

#[cfg(test)]
#[path = "../tests/common/parsing_helpers.rs"]
pub(crate) mod parsing_helpers;

#[cfg(test)]
#[path = "../tests/common/arena_helpers.rs"]
pub(crate) mod arena_helpers;
