use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuplicateKeyBehavior {
    #[default]
    Reject,
    FirstWins,
    LastWins,
}

#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub space_as_plus: bool,
    pub duplicate_keys: DuplicateKeyBehavior,
    pub max_params: Option<usize>,
    pub max_length: Option<usize>,
    pub max_depth: Option<usize>,
}

impl ParseOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn space_as_plus(mut self, enabled: bool) -> Self {
        self.space_as_plus = enabled;
        self
    }

    pub fn duplicate_keys(mut self, behavior: DuplicateKeyBehavior) -> Self {
        self.duplicate_keys = behavior;
        self
    }

    pub fn max_params(mut self, limit: usize) -> Self {
        self.max_params = Some(limit);
        self
    }

    pub fn max_length(mut self, limit: usize) -> Self {
        self.max_length = Some(limit);
        self
    }

    pub fn max_depth(mut self, limit: usize) -> Self {
        self.max_depth = Some(limit);
        self
    }

    pub fn validate(&self) -> Result<(), OptionsValidationError> {
        if matches!(self.max_params, Some(0)) {
            return Err(OptionsValidationError::NonZeroRequired {
                field: "max_params",
            });
        }
        if matches!(self.max_length, Some(0)) {
            return Err(OptionsValidationError::NonZeroRequired {
                field: "max_length",
            });
        }
        if matches!(self.max_depth, Some(0)) {
            return Err(OptionsValidationError::NonZeroRequired { field: "max_depth" });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct StringifyOptions {
    pub space_as_plus: bool,
}

impl StringifyOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn space_as_plus(mut self, enabled: bool) -> Self {
        self.space_as_plus = enabled;
        self
    }

    pub fn validate(&self) -> Result<(), OptionsValidationError> {
        Ok(())
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OptionsValidationError {
    #[error("{field} must be greater than 0 when specified")]
    NonZeroRequired { field: &'static str },
}

#[cfg(test)]
#[path = "options_test.rs"]
mod options_test;
