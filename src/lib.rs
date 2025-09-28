pub mod api;
pub mod prelude;

mod buffer_pool;
mod core;
mod encoding;
mod error;
mod nested;
mod ordered_map;
#[path = "parse.rs"]
mod parse_impl;
mod serde;
mod serde_bridge;
mod serde_impl;
mod stringify;
mod value;

pub(crate) use parse_impl as parse;

pub use api::{
    ParseError, ParseOptions, ParseOptionsBuilder, ParseResult, SerdeStringifyError,
    SerdeStringifyResult, StringifyError, StringifyOptions, StringifyOptionsBuilder,
    StringifyResult, parse, parse_with, set_global_parse_diagnostics, set_global_serde_fastpath,
    stringify, stringify_with,
};

pub use serde::SerdeQueryError;
