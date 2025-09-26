use serde_json::{Map as JsonMap, Value};

#[allow(unused_macros)]
macro_rules! assert_err_matches {
    ($expr:expr, $pattern:pat $(if $guard:expr)? => |$msg:ident| $body:block $(,)?) => {{
        match $expr {
            Ok(_) => panic!(
                "expected {}, but operation succeeded",
                stringify!($pattern $(if $guard)?)
            ),
            Err(err) => {
                let __error_message = err.to_string();
                match err {
                    $pattern $(if $guard)? => {
                        #[allow(unused)]
                        let $msg: &str = &__error_message;
                        $body
                    }
                    other => panic!(
                        "expected {}, got {other:?}",
                        stringify!($pattern $(if $guard)?)
                    ),
                }
            }
        }
    }};
    ($expr:expr, $pattern:pat $(if $guard:expr)? $(,)?) => {{
        match $expr {
            Ok(_) => panic!(
                "expected {}, but operation succeeded",
                stringify!($pattern $(if $guard)?)
            ),
            Err(err) => match err {
                $pattern $(if $guard)? => (),
                other => panic!(
                    "expected {}, got {other:?}",
                    stringify!($pattern $(if $guard)?)
                ),
            },
        }
    }};
}

#[allow(unused_imports)]
pub(crate) use assert_err_matches;

#[track_caller]
pub fn assert_str_entry(map: &JsonMap<String, Value>, key: &str, expected: &str) {
    let value = map
        .get(key)
        .unwrap_or_else(|| panic!("missing key `{key}` in object"));
    match value.as_str() {
        Some(actual) => assert_eq!(actual, expected),
        None => panic!("value for `{key}` was not a string: {value:?}"),
    }
}

#[track_caller]
pub fn expect_object(value: &Value) -> &JsonMap<String, Value> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("expected object value, got {value:?}"))
}

#[track_caller]
pub fn expect_path<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
    let mut current = value;
    for segment in path {
        let object = current
            .as_object()
            .unwrap_or_else(|| panic!("expected object at segment `{segment}`, found {current:?}"));
        current = object
            .get(*segment)
            .unwrap_or_else(|| panic!("missing key `{segment}` while traversing path {path:?}"));
    }
    current
}
