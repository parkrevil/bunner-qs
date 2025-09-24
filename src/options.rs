#[derive(Debug, Clone)]
pub struct ParseOptions {
    pub space_as_plus: bool,
    pub max_params: Option<usize>,
    pub max_length: Option<usize>,
    pub max_depth: Option<usize>,
    pub allow_duplicates: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            space_as_plus: false,
            max_params: None,
            max_length: None,
            max_depth: None,
            allow_duplicates: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StringifyOptions {
    pub space_as_plus: bool,
    pub add_query_prefix: bool,
}
