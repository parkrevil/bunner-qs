use derive_builder::Builder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DuplicateKeyBehavior {
    #[default]
    Reject,
    FirstWins,
    LastWins,
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(pattern = "owned", default, build_fn(validate = "Self::validate"))]
pub struct ParseOptions {
    pub space_as_plus: bool,
    pub duplicate_keys: DuplicateKeyBehavior,
    #[builder(setter(strip_option))]
    pub max_params: Option<usize>,
    #[builder(setter(strip_option))]
    pub max_length: Option<usize>,
    #[builder(setter(strip_option))]
    pub max_depth: Option<usize>,
}

impl ParseOptions {
    pub fn builder() -> ParseOptionsBuilder {
        ParseOptionsBuilder::default()
    }
}

impl ParseOptionsBuilder {
    fn validate(&self) -> Result<(), String> {
        if matches!(self.max_params, Some(Some(0))) {
            return Err("max_params must be greater than 0 when using the builder".into());
        }
        if matches!(self.max_length, Some(Some(0))) {
            return Err("max_length must be greater than 0 when using the builder".into());
        }
        if matches!(self.max_depth, Some(Some(0))) {
            return Err("max_depth must be greater than 0 when using the builder".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(pattern = "owned", default)]
pub struct StringifyOptions {
    pub space_as_plus: bool,
}

impl StringifyOptions {
    pub fn builder() -> StringifyOptionsBuilder {
        StringifyOptionsBuilder::default()
    }
}

#[cfg(test)]
#[path = "options_test.rs"]
mod options_test;
