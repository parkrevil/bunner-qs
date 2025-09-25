use derive_builder::Builder;

#[derive(Debug, Clone, Default, Builder)]
#[builder(pattern = "owned", default)]
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> ParseOptionsBuilder {
        ParseOptionsBuilder::default()
    }
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(pattern = "owned", default)]
pub struct StringifyOptions {
    pub space_as_plus: bool,
}

impl StringifyOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> StringifyOptionsBuilder {
        StringifyOptionsBuilder::default()
    }
}
