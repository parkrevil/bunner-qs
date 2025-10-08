use bunner_qs_rs::{OptionsValidationError, StringifyOptions};

pub fn try_build_stringify_options<F>(
    configure: F,
) -> Result<StringifyOptions, OptionsValidationError>
where
    F: FnOnce(StringifyOptions) -> StringifyOptions,
{
    let options = configure(StringifyOptions::new());
    options.validate()?;
    Ok(options)
}
