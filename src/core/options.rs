use std::sync::atomic::{AtomicBool, Ordering};

/// 전역 serde fast-path 사용 여부를 설정합니다.
pub fn set_global_serde_fastpath(enabled: bool) {
    SERDE_FASTPATH.store(enabled, Ordering::Relaxed);
}

/// 전역 파싱 진단 상세도를 설정합니다.
pub fn set_global_parse_diagnostics(enabled: bool) {
    PARSE_DIAGNOSTICS.store(enabled, Ordering::Relaxed);
}

static SERDE_FASTPATH: AtomicBool = AtomicBool::new(false);
static PARSE_DIAGNOSTICS: AtomicBool = AtomicBool::new(true);

pub(crate) fn global_serde_fastpath() -> bool {
    SERDE_FASTPATH.load(Ordering::Relaxed)
}

pub(crate) fn global_parse_diagnostics() -> bool {
    PARSE_DIAGNOSTICS.load(Ordering::Relaxed)
}

#[derive(Debug, Clone, Default, derive_builder::Builder)]
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

#[derive(Debug, Clone, Default, derive_builder::Builder)]
#[builder(pattern = "owned", default)]
pub struct StringifyOptions {
    pub space_as_plus: bool,
}

impl StringifyOptions {
    pub fn builder() -> StringifyOptionsBuilder {
        StringifyOptionsBuilder::default()
    }
}
