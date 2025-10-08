pub mod api;
mod encode;
pub mod errors;
mod runtime;
mod validate;
mod walker;
mod writer;

pub use api::stringify;
pub use errors::StringifyError;

pub(crate) type StringifyResult<T> = Result<T, errors::StringifyError>;
