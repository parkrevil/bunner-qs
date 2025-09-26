use bunner_qs::{ParseOptions, ParseOptionsBuilder};

pub fn try_build_parse_options<F>(configure: F) -> Result<ParseOptions, String>
where
    F: FnOnce(ParseOptionsBuilder) -> ParseOptionsBuilder,
{
    configure(ParseOptions::builder())
        .build()
        .map_err(|err| err.to_string())
}

pub fn build_parse_options<F>(configure: F) -> ParseOptions
where
    F: FnOnce(ParseOptionsBuilder) -> ParseOptionsBuilder,
{
    try_build_parse_options(configure).expect("parse options builder should succeed")
}
