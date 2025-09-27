use derive_builder::Builder;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Default, Builder)]
#[builder(pattern = "owned", default, build_fn(validate = "Self::validate"))]
pub struct ParseOptions {
    pub space_as_plus: bool,
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

static SERDE_FASTPATH: AtomicBool = AtomicBool::new(false);
static PARSE_DIAGNOSTICS: AtomicBool = AtomicBool::new(true);

#[allow(dead_code)]
pub fn set_global_serde_fastpath(enabled: bool) {
    SERDE_FASTPATH.store(enabled, Ordering::Relaxed);
}

pub(crate) fn global_serde_fastpath() -> bool {
    SERDE_FASTPATH.load(Ordering::Relaxed)
}

#[allow(dead_code)]
pub fn set_global_parse_diagnostics(enabled: bool) {
    PARSE_DIAGNOSTICS.store(enabled, Ordering::Relaxed);
}

pub(crate) fn global_parse_diagnostics() -> bool {
    PARSE_DIAGNOSTICS.load(Ordering::Relaxed)
}
