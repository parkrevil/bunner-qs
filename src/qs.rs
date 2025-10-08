use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::config::OptionsValidationError;
use crate::parsing::{parse, ParseError};
use crate::stringify::{stringify, StringifyError};
use crate::{ParseOptions, StringifyOptions};

#[derive(Debug, Clone, Default)]
pub struct Qs {
    parse: Option<ParseOptions>,
    stringify: Option<StringifyOptions>,
}

impl Qs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parse(mut self, options: ParseOptions) -> Result<Self, OptionsValidationError> {
        options.validate()?;
        self.parse = Some(options);
        Ok(self)
    }

    pub fn with_stringify(
        mut self,
        options: StringifyOptions,
    ) -> Result<Self, OptionsValidationError> {
        options.validate()?;
        self.stringify = Some(options);
        Ok(self)
    }

    pub fn parse<T>(&self, input: impl AsRef<str>) -> Result<T, QsParseError>
    where
        T: DeserializeOwned + Default + 'static,
    {
        let options = self
            .parse
            .as_ref()
            .ok_or(QsParseError::MissingParseOptions)?;
        parse(input, options).map_err(QsParseError::Parse)
    }

    pub fn stringify<T>(&self, data: &T) -> Result<String, QsStringifyError>
    where
        T: Serialize,
    {
        let options = self
            .stringify
            .as_ref()
            .ok_or(QsStringifyError::MissingStringifyOptions)?;
        stringify(data, options).map_err(QsStringifyError::Stringify)
    }

    pub fn parse_options(&self) -> Option<&ParseOptions> {
        self.parse.as_ref()
    }

    pub fn stringify_options(&self) -> Option<&StringifyOptions> {
        self.stringify.as_ref()
    }
}

#[derive(Debug, Error)]
pub enum QsParseError {
    #[error("parse options not configured")]
    MissingParseOptions,
    #[error(transparent)]
    Parse(#[from] ParseError),
}

#[derive(Debug, Error)]
pub enum QsStringifyError {
    #[error("stringify options not configured")]
    MissingStringifyOptions,
    #[error(transparent)]
    Stringify(#[from] StringifyError),
}
