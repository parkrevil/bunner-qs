#![allow(unused)]

mod common;

use bunner_qs::utils::EncodeError;
use bunner_qs::{Charset, Format, QsValue, utils};
use common::{bytes, from_json, make_array, make_object};
use indexmap::IndexMap;
use serde_json::json;
use std::f64;
use std::time::{Duration, UNIX_EPOCH};

fn null_value() -> QsValue {
    QsValue::Null
}

fn undefined_value() -> QsValue {
    QsValue::Undefined
}

fn bool_value(value: bool) -> QsValue {
    QsValue::Bool(value)
}

fn number_value(value: f64) -> QsValue {
    QsValue::Number(value)
}

fn string_value(value: &str) -> QsValue {
    QsValue::String(value.to_string())
}

fn bigint_value(value: &str) -> QsValue {
    QsValue::BigInt(value.to_string())
}

fn null_primitives() -> Vec<QsValue> {
    vec![null_value(), undefined_value()]
}

fn primitives() -> Vec<QsValue> {
    vec![
        null_value(),
        undefined_value(),
        bool_value(true),
        bool_value(false),
        number_value(0.0),
        number_value(42.0),
        number_value(-1.0),
        number_value(f64::NAN),
        number_value(f64::INFINITY),
        number_value(f64::NEG_INFINITY),
        string_value(""),
        string_value("foo"),
        bigint_value("9007199254740991"),
    ]
}

// original: merge()
mod merge {
    use super::*;

    #[test]
    fn merges_true_into_null() {
        let result = utils::merge(null_value(), bool_value(true));
        assert_eq!(result, make_array(vec![null_value(), bool_value(true)]));
    }

    #[test]
    fn merges_null_into_an_array() {
        let result = utils::merge(null_value(), make_array(vec![number_value(42.0)]));
        assert_eq!(result, make_array(vec![null_value(), number_value(42.0)]));
    }

    #[test]
    fn merges_two_objects_with_the_same_key() {
        let result = utils::merge(
            make_object(vec![("a", string_value("b"))]),
            make_object(vec![("a", string_value("c"))]),
        );

        assert_eq!(
            result,
            make_object(vec![(
                "a",
                make_array(vec![string_value("b"), string_value("c")])
            ),]),
        );
    }

    #[test]
    fn merges_a_standalone_and_an_object_into_an_array() {
        let result = utils::merge(
            make_object(vec![("foo", string_value("bar"))]),
            make_object(vec![(
                "foo",
                make_object(vec![("first", string_value("123"))]),
            )]),
        );

        assert_eq!(
            result,
            make_object(vec![(
                "foo",
                make_array(vec![
                    string_value("bar"),
                    make_object(vec![("first", string_value("123"))]),
                ]),
            ),]),
        );
    }

    #[test]
    fn merges_a_standalone_and_two_objects_into_an_array() {
        let result = utils::merge(
            make_object(vec![(
                "foo",
                make_array(vec![
                    string_value("bar"),
                    make_object(vec![("first", string_value("123"))]),
                ]),
            )]),
            make_object(vec![(
                "foo",
                make_object(vec![("second", string_value("456"))]),
            )]),
        );

        let mut expected_inner = IndexMap::new();
        expected_inner.insert("0".to_string(), string_value("bar"));
        expected_inner.insert(
            "1".to_string(),
            make_object(vec![("first", string_value("123"))]),
        );
        expected_inner.insert("second".to_string(), string_value("456"));

        assert_eq!(
            result,
            make_object(vec![("foo", QsValue::Object(expected_inner))]),
        );
    }

    #[test]
    fn merges_an_object_sandwiched_by_two_standalones_into_an_array() {
        let result = utils::merge(
            make_object(vec![(
                "foo",
                make_array(vec![
                    string_value("bar"),
                    make_object(vec![
                        ("first", string_value("123")),
                        ("second", string_value("456")),
                    ]),
                ]),
            )]),
            make_object(vec![("foo", string_value("baz"))]),
        );

        assert_eq!(
            result,
            make_object(vec![(
                "foo",
                make_array(vec![
                    string_value("bar"),
                    make_object(vec![
                        ("first", string_value("123")),
                        ("second", string_value("456")),
                    ]),
                    string_value("baz"),
                ]),
            ),]),
        );
    }

    #[test]
    fn merges_nested_arrays() {
        let result = utils::merge(
            make_object(vec![("foo", make_array(vec![string_value("baz")]))]),
            make_object(vec![(
                "foo",
                make_array(vec![string_value("bar"), string_value("xyzzy")]),
            )]),
        );

        assert_eq!(
            result,
            make_object(vec![(
                "foo",
                make_array(vec![
                    string_value("baz"),
                    string_value("bar"),
                    string_value("xyzzy"),
                ]),
            ),]),
        );
    }

    #[test]
    fn merges_non_object_source_into_an_object() {
        let result = utils::merge(
            make_object(vec![("foo", string_value("baz"))]),
            string_value("bar"),
        );

        assert_eq!(
            result,
            make_object(vec![
                ("foo", string_value("baz")),
                ("bar", bool_value(true)),
            ]),
        );
    }

    // function merge tests removed: Function type unsupported in Rust port

    #[test]
    fn avoids_invoking_array_setters_unnecessarily() {
        let observed = make_array(vec![make_object(vec![("bar", string_value("baz"))])]);
        let observed_snapshot = observed.clone();
        let result = utils::merge(observed.clone(), make_array(vec![null_value()]));

        assert_eq!(
            observed, observed_snapshot,
            "merge should not mutate the original array"
        );
        assert_eq!(
            result,
            make_array(vec![
                make_object(vec![("bar", string_value("baz"))]),
                null_value(),
            ]),
        );
    }
}

// original: assign()
mod assign {
    use super::*;

    #[test]
    fn assigns_properties_into_target() {
        let mut target = from_json(json!({ "a": 1, "b": 2 }));
        let source = from_json(json!({ "b": 3, "c": 4 }));
        let source_snapshot = source.clone();

        let result = utils::assign(&mut target, &source);

        let expected = from_json(json!({ "a": 1, "b": 3, "c": 4 }));
        assert_eq!(result, expected, "returns the target");
        assert_eq!(target, expected, "target and source are merged");
        assert_eq!(source, source_snapshot, "source is untouched");
    }
}

// original: combine()
mod combine {
    use super::*;

    // original: both arrays
    #[test]
    fn both_arrays() {
        let a = make_array(vec![number_value(1.0)]);
        let b = make_array(vec![number_value(2.0)]);

        let result = utils::combine(a.clone(), b.clone());

        assert_eq!(a, make_array(vec![number_value(1.0)]), "a is not mutated");
        assert_eq!(b, make_array(vec![number_value(2.0)]), "b is not mutated");
        assert_eq!(
            result,
            make_array(vec![number_value(1.0), number_value(2.0)]),
            "combined is a + b",
        );
    }

    // original: one array, one non-array
    #[test]
    fn one_array_one_non_array() {
        let a_non = number_value(1.0);
        let a = make_array(vec![a_non.clone()]);
        let b_non = number_value(2.0);
        let b = make_array(vec![b_non.clone()]);

        let combined_a_non_b = utils::combine(a_non.clone(), b.clone());
        assert_eq!(b, make_array(vec![b_non.clone()]), "b is not mutated");
        assert_eq!(
            combined_a_non_b,
            make_array(vec![number_value(1.0), number_value(2.0)]),
            "first argument is array-wrapped when not an array",
        );

        let combined_a_b_non = utils::combine(a.clone(), b_non.clone());
        assert_eq!(a, make_array(vec![a_non.clone()]), "a is not mutated");
        assert_eq!(
            combined_a_b_non,
            make_array(vec![number_value(1.0), number_value(2.0)]),
            "second argument is array-wrapped when not an array",
        );
    }

    // original: neither is an array
    #[test]
    fn neither_is_an_array() {
        let result = utils::combine(number_value(1.0), number_value(2.0));
        assert_eq!(
            result,
            make_array(vec![number_value(1.0), number_value(2.0)]),
            "both arguments are array-wrapped when not an array",
        );
    }
}

// original: decode
mod decode {
    use super::*;

    #[test]
    fn decodes_plus_to_space() {
        let result = utils::decode("a+b", None).expect("decode result");
        assert_eq!(result, "a b");
    }

    #[test]
    fn decodes_a_string() {
        let result = utils::decode("name%2Eobj", None).expect("decode result");
        assert_eq!(result, "name.obj");
    }

    #[test]
    fn decodes_a_string_in_iso_8859_1() {
        let result =
            utils::decode("name%2Eobj%2Efoo", Some(Charset::Iso88591)).expect("decode result");
        assert_eq!(result, "name.obj.foo");
    }
}

// original: encode
mod encode {
    use super::*;

    fn encode_ok(value: &QsValue, charset: Option<Charset>, format: Option<Format>) -> QsValue {
        utils::encode(value, charset, format).expect("encode result")
    }

    #[test]
    fn rejects_null_primitives() {
        for nullish in null_primitives() {
            match utils::encode(&nullish, None, None) {
                Err(EncodeError::TypeError(message)) => {
                    assert!(
                        message.contains("is not a string"),
                        "unexpected error message: {message}"
                    );
                }
                Err(other) => panic!("expected TypeError, got {other:?}"),
                Ok(_) => panic!("{nullish:?} should not encode successfully"),
            }
        }
    }

    #[test]
    fn empty_string_returns_itself() {
        let input = string_value("");
        let result = encode_ok(&input, None, None);
        assert_eq!(result, string_value(""));
    }

    #[test]
    fn empty_array_returns_itself() {
        let input = make_array(vec![]);
        let result = encode_ok(&input, None, None);
        assert_eq!(result, make_array(vec![]));
    }

    #[test]
    fn empty_arraylike_returns_itself() {
        let input = make_object(vec![("length", number_value(0.0))]);
        let result = encode_ok(&input, None, None);
        assert_eq!(result, make_object(vec![("length", number_value(0.0))]));
    }

    // symbols test removed: Symbol unsupported in Rust port

    #[test]
    fn encodes_parentheses() {
        let result = encode_ok(&string_value("(abc)"), None, None);
        assert_eq!(result, string_value("%28abc%29"));
    }

    #[test]
    fn to_strings_and_encodes_parentheses() {
        let input = QsValue::Custom("(abc)".to_string());
        let result = encode_ok(&input, None, None);
        assert_eq!(result, string_value("%28abc%29"));
    }

    #[test]
    fn encodes_in_iso_8859_1() {
        let input = string_value("abc 123 💩");
        let result = encode_ok(&input, Some(Charset::Iso88591), None);
        assert_eq!(
            result,
            string_value("abc%20123%20%26%2355357%3B%26%2356489%3B"),
        );
    }

    #[test]
    fn encodes_a_long_string() {
        let long_string: String = (0..1500).map(|_| " ").collect();
        let expected: String = (0..1500).map(|_| "%20").collect();
        let result = encode_ok(&string_value(&long_string), None, None);
        assert_eq!(result, string_value(&expected));
    }

    #[test]
    fn encodes_parens_normally() {
        let result = encode_ok(&string_value("\u{0028}\u{0029}"), None, None);
        assert_eq!(result, string_value("%28%29"));
    }

    #[test]
    fn does_not_encode_parens_in_rfc1738() {
        let result = encode_ok(
            &string_value("\u{0028}\u{0029}"),
            None,
            Some(Format::Rfc1738),
        );
        assert_eq!(result, string_value("()"));
    }

    // todo RFC1738 format

    #[test]
    fn encodes_multibyte_chars() {
        let result = encode_ok(&string_value("Āက豈"), None, None);
        assert_eq!(result, string_value("%C4%80%E1%80%80%EF%A4%80"));
    }

    #[test]
    #[ignore = "Requires handling of lone surrogate code points"]
    fn encodes_lone_surrogates() {
        unimplemented!("encodes_lone_surrogates");
    }
}

// original: isBuffer()
mod isbuffer {
    use super::*;

    #[test]
    fn detects_buffers() {
        let sample_values = vec![
            null_value(),
            undefined_value(),
            bool_value(true),
            bool_value(false),
            string_value(""),
            string_value("abc"),
            number_value(42.0),
            number_value(0.0),
            number_value(f64::NAN),
            make_object(vec![]),
            make_array(vec![]),
            QsValue::Custom("function".to_string()),
        ];

        for value in sample_values {
            assert!(!utils::is_buffer(&value), "{value:?} is not a buffer");
        }

        let fake_buffer = make_object(vec![("constructor", string_value("Buffer"))]);
        assert!(
            !utils::is_buffer(&fake_buffer),
            "fake buffer is not a buffer"
        );

        let safer_buffer = bytes(b"abc");
        assert!(
            utils::is_buffer(&safer_buffer),
            "SaferBuffer instance is a buffer"
        );

        let buffer = bytes(b"abc");
        assert!(
            utils::is_buffer(&buffer),
            "real Buffer instance is a buffer"
        );
    }
}

// isRegExp tests removed: RegExp unsupported in Rust port

// function_value helper removed
