use bunner_qs_rs::{ParseOptions, ParseOptionsBuilder};

pub fn try_build_parse_options<F>(configure: F) -> Result<ParseOptions, String>
where
    F: FnOnce(ParseOptionsBuilder) -> ParseOptionsBuilder,
{
    configure(ParseOptions::builder())
        .build()
        .map_err(|err| err.to_string())
}
