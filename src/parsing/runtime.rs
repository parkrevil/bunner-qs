use crate::config::{ParseOptions, global_parse_diagnostics, global_serde_fastpath};

#[derive(Clone, Copy)]
pub(crate) struct ParseRuntime {
    pub(crate) space_as_plus: bool,
    pub(crate) max_params: Option<usize>,
    pub(crate) max_length: Option<usize>,
    pub(crate) max_depth: Option<usize>,
    pub(crate) diagnostics: bool,
    pub(crate) serde_fastpath: bool,
}

impl ParseRuntime {
    pub(crate) fn new(options: &ParseOptions) -> Self {
        Self {
            space_as_plus: options.space_as_plus,
            max_params: options.max_params,
            max_length: options.max_length,
            max_depth: options.max_depth,
            diagnostics: global_parse_diagnostics(),
            serde_fastpath: global_serde_fastpath(),
        }
    }
}
