use bunner_qs_rs::{StringifyOptions, StringifyOptionsBuilder};

pub fn try_build_stringify_options<F>(configure: F) -> Result<StringifyOptions, String>
where
    F: FnOnce(StringifyOptionsBuilder) -> StringifyOptionsBuilder,
{
    configure(StringifyOptions::builder())
        .build()
        .map_err(|err| err.to_string())
}
