use bunner_qs_rs::{OptionsValidationError, ParseOptions};

pub fn try_build_parse_options<F>(configure: F) -> Result<ParseOptions, OptionsValidationError>
where
    F: FnOnce(ParseOptions) -> ParseOptions,
{
    let options = configure(ParseOptions::new());
    options.validate()?;
    Ok(options)
}
