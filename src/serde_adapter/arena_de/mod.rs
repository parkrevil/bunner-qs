pub mod value_ref;
pub mod deserializer;

pub use deserializer::*;

fn format_expected(fields: &'static [&'static str]) -> String {
    if fields.is_empty() {
        "(none)".into()
    } else {
        fields.join(", ")
    }
}