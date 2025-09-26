use bunner_qs::{StringifyOptions, StringifyOptionsBuilder};

pub fn build_stringify_options<F>(configure: F) -> StringifyOptions
where
    F: FnOnce(StringifyOptionsBuilder) -> StringifyOptionsBuilder,
{
    configure(StringifyOptions::builder())
        .build()
        .map_err(|err| err.to_string())
        .expect("stringify options builder should succeed")
}
