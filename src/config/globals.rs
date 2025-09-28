use std::sync::atomic::{AtomicBool, Ordering};

static SERDE_FASTPATH: AtomicBool = AtomicBool::new(false);
static PARSE_DIAGNOSTICS: AtomicBool = AtomicBool::new(true);

pub fn set_global_serde_fastpath(enabled: bool) {
    SERDE_FASTPATH.store(enabled, Ordering::Relaxed);
}

pub fn set_global_parse_diagnostics(enabled: bool) {
    PARSE_DIAGNOSTICS.store(enabled, Ordering::Relaxed);
}

pub(crate) fn global_serde_fastpath() -> bool {
    SERDE_FASTPATH.load(Ordering::Relaxed)
}

pub(crate) fn global_parse_diagnostics() -> bool {
    PARSE_DIAGNOSTICS.load(Ordering::Relaxed)
}
