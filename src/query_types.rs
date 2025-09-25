use crate::QueryMap;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TypedValueError {
    #[error("key '{key}' is missing")]
    Missing { key: String },
    #[error("key '{key}' has no values")]
    Empty { key: String },
    #[error("key '{key}' has {count} values; expected a single value")]
    Multiple { key: String, count: usize },
    #[error("failed to parse value '{value}' for key '{key}': {message}")]
    Parse {
        key: String,
        value: String,
        message: String,
    },
    #[error("invalid boolean literal '{value}' for key '{key}'")]
    InvalidBool { key: String, value: String },
}

pub trait QueryTypesExt {
    fn first(&self, key: &str) -> Option<&str>;
    fn require_first(&self, key: &str) -> Result<&str, TypedValueError>;
    fn parse_first<T>(&self, key: &str) -> Result<Option<T>, TypedValueError>
    where
        T: FromStr,
        T::Err: Display;
    fn parse_required<T>(&self, key: &str) -> Result<T, TypedValueError>
    where
        T: FromStr,
        T::Err: Display;
    fn parse_all<T>(&self, key: &str) -> Result<Vec<T>, TypedValueError>
    where
        T: FromStr,
        T::Err: Display;
    fn bool(&self, key: &str) -> Result<Option<bool>, TypedValueError>;
    fn bool_required(&self, key: &str) -> Result<bool, TypedValueError>;
    fn i64(&self, key: &str) -> Result<Option<i64>, TypedValueError>;
    fn u64(&self, key: &str) -> Result<Option<u64>, TypedValueError>;
    fn f64(&self, key: &str) -> Result<Option<f64>, TypedValueError>;
}

impl QueryTypesExt for QueryMap {
    fn first(&self, key: &str) -> Option<&str> {
        self.get(key)
            .and_then(|values| values.first().map(|value| value.as_str()))
    }

    fn require_first(&self, key: &str) -> Result<&str, TypedValueError> {
        match self.first(key) {
            Some(value) => Ok(value),
            None => Err(TypedValueError::Missing {
                key: key.to_string(),
            }),
        }
    }

    fn parse_first<T>(&self, key: &str) -> Result<Option<T>, TypedValueError>
    where
        T: FromStr,
        T::Err: Display,
    {
        let values = match self.get(key) {
            Some(values) => values,
            None => return Ok(None),
        };

        match values.as_slice() {
            [] => Err(TypedValueError::Empty {
                key: key.to_string(),
            }),
            [single] => single
                .parse::<T>()
                .map(Some)
                .map_err(|err| TypedValueError::Parse {
                    key: key.to_string(),
                    value: single.clone(),
                    message: err.to_string(),
                }),
            slice => Err(TypedValueError::Multiple {
                key: key.to_string(),
                count: slice.len(),
            }),
        }
    }

    fn parse_required<T>(&self, key: &str) -> Result<T, TypedValueError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.parse_first(key)?
            .ok_or_else(|| TypedValueError::Missing {
                key: key.to_string(),
            })
    }

    fn parse_all<T>(&self, key: &str) -> Result<Vec<T>, TypedValueError>
    where
        T: FromStr,
        T::Err: Display,
    {
        let values = match self.get(key) {
            Some(values) => values,
            None => return Ok(Vec::new()),
        };

        let mut parsed = Vec::with_capacity(values.len());
        for value in values {
            parsed.push(value.parse::<T>().map_err(|err| TypedValueError::Parse {
                key: key.to_string(),
                value: value.clone(),
                message: err.to_string(),
            })?);
        }

        Ok(parsed)
    }

    fn bool(&self, key: &str) -> Result<Option<bool>, TypedValueError> {
        let raw = match self.get(key) {
            Some(values) => values,
            None => return Ok(None),
        };

        match raw.as_slice() {
            [] => Err(TypedValueError::Empty {
                key: key.to_string(),
            }),
            [single] => parse_bool(key, single).map(Some),
            slice => Err(TypedValueError::Multiple {
                key: key.to_string(),
                count: slice.len(),
            }),
        }
    }

    fn bool_required(&self, key: &str) -> Result<bool, TypedValueError> {
        self.bool(key)?.ok_or_else(|| TypedValueError::Missing {
            key: key.to_string(),
        })
    }

    fn i64(&self, key: &str) -> Result<Option<i64>, TypedValueError> {
        self.parse_first(key)
    }

    fn u64(&self, key: &str) -> Result<Option<u64>, TypedValueError> {
        self.parse_first(key)
    }

    fn f64(&self, key: &str) -> Result<Option<f64>, TypedValueError> {
        self.parse_first(key)
    }
}

fn parse_bool(key: &str, value: &str) -> Result<bool, TypedValueError> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "on" | "yes" => Ok(true),
        "false" | "0" | "off" | "no" => Ok(false),
        other => Err(TypedValueError::InvalidBool {
            key: key.to_string(),
            value: other.to_string(),
        }),
    }
}
