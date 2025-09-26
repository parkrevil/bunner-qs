mod errors {
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

    pub(crate) use assert_err_matches;
}

pub(crate) mod paths {
    use serde_json::Value;

    #[track_caller]
    pub fn expect_path<'a>(value: &'a Value, path: &[&str]) -> &'a Value {
        let mut current = value;
        for segment in path {
            let object = current.as_object().unwrap_or_else(|| {
                panic!("expected object at segment `{segment}`, found {current:?}")
            });
            current = object.get(*segment).unwrap_or_else(|| {
                panic!("missing key `{segment}` while traversing path {path:?}")
            });
        }
        current
    }

    #[track_caller]
    pub fn assert_str_path(value: &Value, path: &[&str], expected: &str) {
        let node = expect_path(value, path);
        match node.as_str() {
            Some(actual) => assert_eq!(actual, expected, "string at path {path:?} did not match"),
            None => panic!("value at path {path:?} was not a string: {node:?}"),
        }
    }
}

pub(crate) mod arrays {
    use super::paths::expect_path;
    use serde_json::Value;

    #[track_caller]
    fn assert_string_array(value: &Value, expected: &[&str]) {
        match value.as_array() {
            Some(items) => {
                assert_eq!(items.len(), expected.len(), "array length mismatch");
                for (idx, expected_value) in expected.iter().enumerate() {
                    let actual = items[idx]
                        .as_str()
                        .unwrap_or_else(|| panic!("array index {idx} not a string"));
                    assert_eq!(
                        actual, *expected_value,
                        "array value mismatch at index {idx}"
                    );
                }
            }
            None => panic!("expected array value, got {value:?}"),
        }
    }

    #[track_caller]
    pub(crate) fn assert_string_array_path(value: &Value, path: &[&str], expected: &[&str]) {
        let node = expect_path(value, path);
        assert_string_array(node, expected);
    }

    const _: fn(&Value, &[&str], &[&str]) = assert_string_array_path;
}

#[allow(unused_imports)]
pub(crate) use arrays::assert_string_array_path;
#[allow(unused_imports)]
pub(crate) use errors::assert_err_matches;
#[allow(unused_imports)]
pub(crate) use paths::{assert_str_path, expect_path};
