pub mod api;
mod encode;
pub mod errors;
mod runtime;
mod validate;
mod walker;
mod writer;

pub(crate) type StringifyResult<T> = Result<T, errors::StringifyError>;
