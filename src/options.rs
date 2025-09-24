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

#[derive(Debug, Clone)]
pub struct ParseOptionsBuilder {
    inner: ParseOptions,
}

impl ParseOptions {
    pub fn builder() -> ParseOptionsBuilder {
        ParseOptionsBuilder {
            inner: ParseOptions::default(),
        }
    }
}

impl ParseOptionsBuilder {
    pub fn space_as_plus(mut self, enabled: bool) -> Self {
        self.inner.space_as_plus = enabled;
        self
    }

    pub fn max_params(mut self, limit: Option<usize>) -> Self {
        self.inner.max_params = limit;
        self
    }

    pub fn max_length(mut self, limit: Option<usize>) -> Self {
        self.inner.max_length = limit;
        self
    }

    pub fn max_depth(mut self, limit: Option<usize>) -> Self {
        self.inner.max_depth = limit;
        self
    }

    pub fn allow_duplicates(mut self, allow: bool) -> Self {
        self.inner.allow_duplicates = allow;
        self
    }

    pub fn build(self) -> ParseOptions {
        self.inner
    }
}

#[derive(Debug, Clone, Default)]
pub struct StringifyOptions {
    pub space_as_plus: bool,
    pub add_query_prefix: bool,
}

#[derive(Debug, Clone)]
pub struct StringifyOptionsBuilder {
    inner: StringifyOptions,
}

impl StringifyOptions {
    pub fn builder() -> StringifyOptionsBuilder {
        StringifyOptionsBuilder {
            inner: StringifyOptions::default(),
        }
    }
}

impl StringifyOptionsBuilder {
    pub fn space_as_plus(mut self, enabled: bool) -> Self {
        self.inner.space_as_plus = enabled;
        self
    }

    pub fn add_query_prefix(mut self, enabled: bool) -> Self {
        self.inner.add_query_prefix = enabled;
        self
    }

    pub fn build(self) -> StringifyOptions {
        self.inner
    }
}
