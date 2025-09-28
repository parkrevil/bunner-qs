use crate::config::ParseOptions;

#[derive(Clone, Copy)]
pub(crate) struct ParseRuntime {
    pub(crate) space_as_plus: bool,
    pub(crate) max_params: Option<usize>,
    pub(crate) max_length: Option<usize>,
    pub(crate) max_depth: Option<usize>,
}

impl ParseRuntime {
    pub(crate) fn new(options: &ParseOptions) -> Self {
        Self {
            space_as_plus: options.space_as_plus,
            max_params: options.max_params,
            max_length: options.max_length,
            max_depth: options.max_depth,
        }
    }
}
