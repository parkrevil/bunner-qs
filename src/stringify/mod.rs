mod api;
mod encode;
mod errors;
mod runtime;
mod validate;
mod walker;
mod writer;

pub use self::errors::{SerdeStringifyError, StringifyError};
pub use api::{stringify, stringify_with};

pub type StringifyResult<T> = Result<T, StringifyError>;

pub type SerdeStringifyResult<T> = Result<T, SerdeStringifyError>;

// No tests here; public API tests live in api.rs (api_test.rs)
