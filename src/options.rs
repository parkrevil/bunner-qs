#[derive(Debug, Clone, Default)]
pub struct ParseOptions {
    pub space_as_plus: bool,
    pub max_params: Option<usize>,
    pub max_length: Option<usize>,
    pub max_depth: Option<usize>,
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
