//! Auto-generated skeleton from qs/test/stringify.js
#![allow(unused)]

mod common;

use bunner_qs::{
    ArrayFormat, Charset, EncodeFn, Filter, Format, QsValue, StringifyError, StringifyOptions,
    ValueKind, stringify, stringify_with_options,
};
use common::{
    build_stringify_options, from_json, make_array, make_object, stringify_default, stringify_with,
};
use serde_json::json;
use std::mem;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn expect_ok(value: QsValue, expected: &str) {
    let result = stringify_default(&value);
    match result {
        Ok(actual) => assert_eq!(
            actual, expected,
            "unexpected stringify output for {value:?}"
        ),
        Err(error) => panic!("stringify returned error for {value:?}: {error:?}"),
    }
}

fn expect_with(value: QsValue, configure: impl FnOnce(&mut StringifyOptions), expected: &str) {
    let mut options = StringifyOptions::default();
    configure(&mut options);
    let result = stringify_with(&value, options);
    match result {
        Ok(actual) => assert_eq!(
            actual, expected,
            "unexpected stringify output for {value:?}"
        ),
        Err(error) => panic!("stringify returned error for {value:?}: {error:?}"),
    }
}

fn expect_err(value: QsValue, configure: impl FnOnce(&mut StringifyOptions)) {
    let mut options = StringifyOptions::default();
    configure(&mut options);
    let result = stringify_with(&value, options);
    assert!(
        result.is_err(),
        "expected stringify to return an error for {value:?}"
    );
}

fn expect_err_default(value: QsValue) {
    let result = stringify_default(&value);
    assert!(
        result.is_err(),
        "expected stringify to return an error for {value:?}"
    );
}

fn qs_value(value: serde_json::Value) -> QsValue {
    from_json(value)
}

fn symbol(name: &str) -> QsValue {
    QsValue::Symbol(name.to_string())
}

fn bigint(value: &str) -> QsValue {
    QsValue::BigInt(value.to_string())
}

fn bytes(data: &[u8]) -> QsValue {
    QsValue::Bytes(data.to_vec())
}

fn date_from_millis(millis: u64) -> QsValue {
    QsValue::Date(UNIX_EPOCH + Duration::from_millis(millis))
}

// original: stringify()
mod stringify {
    use super::*;

    // original: stringifies a querystring object
    #[test]
    fn stringifies_a_querystring_object() {
        expect_ok(qs_value(json!({ "a": "b" })), "a=b");
        expect_ok(qs_value(json!({ "a": 1 })), "a=1");
        expect_ok(qs_value(json!({ "a": 1, "b": 2 })), "a=1&b=2");
        expect_ok(qs_value(json!({ "a": "A_Z" })), "a=A_Z");
        expect_ok(qs_value(json!({ "a": "€" })), "a=%E2%82%AC");
        expect_ok(qs_value(json!({ "a": "\u{E000}" })), "a=%EE%80%80");
        expect_ok(qs_value(json!({ "a": "א" })), "a=%D7%90");
        expect_ok(qs_value(json!({ "a": "\u{10437}" })), "a=%F0%90%90%B7");
    }

    // original: stringifies falsy values
    #[test]
    fn stringifies_falsy_values() {
        expect_ok(QsValue::Undefined, "");
        expect_ok(QsValue::Null, "");
        expect_with(
            QsValue::Null,
            |opts| {
                opts.strict_null_handling = true;
            },
            "",
        );
        expect_ok(QsValue::Bool(false), "");
        expect_ok(QsValue::Number(0.0), "");
    }

    // original: stringifies symbols
    #[test]
    fn stringifies_symbols() {
        let sym = symbol("Symbol(Symbol.iterator)");
        expect_ok(sym.clone(), "");
        expect_ok(
            make_array(vec![sym.clone()]),
            "0=Symbol%28Symbol.iterator%29",
        );
        expect_ok(
            make_object(vec![("a", sym.clone())]),
            "a=Symbol%28Symbol.iterator%29",
        );
        expect_with(
            make_object(vec![("a", make_array(vec![sym]))]),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=Symbol%28Symbol.iterator%29",
        );
    }

    // original: stringifies bigints
    #[test]
    fn stringifies_bigints() {
        let three = bigint("3");
        let encode_with_n = Arc::new(
            |value: &str,
             default_encoder: &dyn Fn(&str, Charset, ValueKind) -> String,
             charset: Charset,
             kind: ValueKind| {
                let encoded = default_encoder(value, charset, kind);
                if matches!(kind, ValueKind::Value) {
                    format!("{encoded}n")
                } else {
                    encoded
                }
            },
        );

        expect_ok(three.clone(), "");
        expect_ok(make_array(vec![three.clone()]), "0=3");
        expect_with(
            make_array(vec![three.clone()]),
            |opts| {
                opts.encoder = Some(encode_with_n.clone());
            },
            "0=3n",
        );
        expect_ok(make_object(vec![("a", three.clone())]), "a=3");
        expect_with(
            make_object(vec![("a", three.clone())]),
            |opts| {
                opts.encoder = Some(encode_with_n.clone());
            },
            "a=3n",
        );
        expect_with(
            make_object(vec![("a", make_array(vec![three.clone()]))]),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=3",
        );
        expect_with(
            make_object(vec![("a", make_array(vec![three]))]),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
                opts.encoder = Some(encode_with_n);
            },
            "a[]=3n",
        );
    }

    // original: encodes dot in key of object when encodeDotInKeys and allowDots is provided
    #[test]
    fn encodes_dot_in_key_of_object_when_encodedotinkeys_and_allowdots_is_provided() {
        let value = qs_value(json!({ "name.obj": { "first": "John", "last": "Doe" } }));

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = false;
                opts.encode_dot_in_keys = false;
            },
            "name.obj%5Bfirst%5D=John&name.obj%5Blast%5D=Doe",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode_dot_in_keys = false;
            },
            "name.obj.first=John&name.obj.last=Doe",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = false;
                opts.encode_dot_in_keys = true;
            },
            "name%252Eobj%5Bfirst%5D=John&name%252Eobj%5Blast%5D=Doe",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode_dot_in_keys = true;
            },
            "name%252Eobj.first=John&name%252Eobj.last=Doe",
        );

        let nested = qs_value(json!({
            "name.obj.subobject": {
                "first.godly.name": "John",
                "last": "Doe"
            }
        }));

        expect_with(
            nested.clone(),
            |opts| {
                opts.allow_dots = false;
                opts.encode_dot_in_keys = false;
            },
            "name.obj.subobject%5Bfirst.godly.name%5D=John&name.obj.subobject%5Blast%5D=Doe",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode_dot_in_keys = false;
            },
            "name.obj.subobject.first.godly.name=John&name.obj.subobject.last=Doe",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.allow_dots = false;
                opts.encode_dot_in_keys = true;
            },
            "name%252Eobj%252Esubobject%5Bfirst.godly.name%5D=John&name%252Eobj%252Esubobject%5Blast%5D=Doe",
        );

        expect_with(
            nested,
            |opts| {
                opts.allow_dots = true;
                opts.encode_dot_in_keys = true;
            },
            "name%252Eobj%252Esubobject.first%252Egodly%252Ename=John&name%252Eobj%252Esubobject.last=Doe",
        );
    }

    // original: should encode dot in key of object, and automatically set allowDots to `true` when encodeDotInKeys is true and allowDots in undefined
    #[test]
    fn should_encode_dot_in_key_of_object_and_automatically_set_allowdots_to_true_when_encodedotinkeys_is_true_and_allowdots_in_undefined()
     {
        let value = qs_value(json!({
            "name.obj.subobject": {
                "first.godly.name": "John",
                "last": "Doe"
            }
        }));

        expect_with(
            value,
            |opts| {
                opts.encode_dot_in_keys = true;
            },
            "name%252Eobj%252Esubobject.first%252Egodly%252Ename=John&name%252Eobj%252Esubobject.last=Doe",
        );
    }

    // original: should encode dot in key of object when encodeDotInKeys and allowDots is provided, and nothing else when encodeValuesOnly is provided
    #[test]
    fn should_encode_dot_in_key_of_object_when_encodedotinkeys_and_allowdots_is_provided_and_nothing_else_when_encodevaluesonly_is_provided()
     {
        let value = qs_value(json!({ "name.obj": { "first": "John", "last": "Doe" } }));
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_dot_in_keys = true;
                opts.allow_dots = true;
                opts.encode_values_only = true;
            },
            "name%2Eobj.first=John&name%2Eobj.last=Doe",
        );

        let nested = qs_value(json!({
            "name.obj.subobject": {
                "first.godly.name": "John",
                "last": "Doe"
            }
        }));
        expect_with(
            nested,
            |opts| {
                opts.encode_dot_in_keys = true;
                opts.allow_dots = true;
                opts.encode_values_only = true;
            },
            "name%2Eobj%2Esubobject.first%2Egodly%2Ename=John&name%2Eobj%2Esubobject.last=Doe",
        );
    }

    // original: throws when `commaRoundTrip` is not a boolean
    #[test]
    fn throws_when_commaroundtrip_is_not_a_boolean() {
        expect_err(make_object(vec![]), |opts| {
            opts.additional.insert(
                "commaRoundTrip".to_string(),
                QsValue::String("not a boolean".to_string()),
            );
        });
    }

    // original: throws when `encodeDotInKeys` is not a boolean
    #[test]
    fn throws_when_encodedotinkeys_is_not_a_boolean() {
        let value = qs_value(json!({ "a": [], "b": "zz" }));
        let invalid_values = vec![
            QsValue::String("foobar".to_string()),
            QsValue::Number(0.0),
            QsValue::Number(f64::NAN),
            QsValue::Null,
        ];

        for invalid in invalid_values {
            expect_err(value.clone(), |opts| {
                opts.additional
                    .insert("encodeDotInKeys".to_string(), invalid.clone());
            });
        }
    }

    // original: adds query prefix
    #[test]
    fn adds_query_prefix() {
        expect_with(
            qs_value(json!({ "a": "b" })),
            |opts| {
                opts.add_query_prefix = true;
            },
            "?a=b",
        );
    }

    // original: with query prefix, outputs blank string given an empty object
    #[test]
    fn with_query_prefix_outputs_blank_string_given_an_empty_object() {
        expect_with(
            make_object(vec![]),
            |opts| {
                opts.add_query_prefix = true;
            },
            "",
        );
    }

    // original: stringifies nested falsy values
    #[test]
    fn stringifies_nested_falsy_values() {
        expect_ok(
            qs_value(json!({ "a": { "b": { "c": null } } })),
            "a%5Bb%5D%5Bc%5D=",
        );
        expect_with(
            qs_value(json!({ "a": { "b": { "c": null } } })),
            |opts| opts.strict_null_handling = true,
            "a%5Bb%5D%5Bc%5D",
        );
        expect_ok(
            qs_value(json!({ "a": { "b": { "c": false } } })),
            "a%5Bb%5D%5Bc%5D=false",
        );
    }

    // original: stringifies a nested object
    #[test]
    fn stringifies_a_nested_object() {
        expect_ok(qs_value(json!({ "a": { "b": "c" } })), "a%5Bb%5D=c");
        expect_ok(
            qs_value(json!({ "a": { "b": { "c": { "d": "e" } } } })),
            "a%5Bb%5D%5Bc%5D%5Bd%5D=e",
        );
    }

    // original: `allowDots` option: stringifies a nested object with dots notation
    #[test]
    fn allowdots_option_stringifies_a_nested_object_with_dots_notation() {
        expect_with(
            qs_value(json!({ "a": { "b": "c" } })),
            |opts| opts.allow_dots = true,
            "a.b=c",
        );
        expect_with(
            qs_value(json!({ "a": { "b": { "c": { "d": "e" } } } })),
            |opts| opts.allow_dots = true,
            "a.b.c.d=e",
        );
    }

    // original: stringifies an array value
    #[test]
    fn stringifies_an_array_value() {
        let value = qs_value(json!({ "a": ["b", "c", "d"] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Indices;
            },
            "a%5B0%5D=b&a%5B1%5D=c&a%5B2%5D=d",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Brackets;
            },
            "a%5B%5D=b&a%5B%5D=c&a%5B%5D=d",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Comma;
            },
            "a=b%2Cc%2Cd",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a=b%2Cc%2Cd",
        );

        expect_ok(value, "a%5B0%5D=b&a%5B1%5D=c&a%5B2%5D=d");
    }

    // original: `skipNulls` option
    #[test]
    fn skipnulls_option() {
        expect_with(
            qs_value(json!({ "a": "b", "c": null })),
            |opts| opts.skip_nulls = true,
            "a=b",
        );

        expect_with(
            qs_value(json!({ "a": { "b": "c", "d": null } })),
            |opts| opts.skip_nulls = true,
            "a%5Bb%5D=c",
        );
    }

    // original: omits array indices when asked
    #[test]
    fn omits_array_indices_when_asked() {
        expect_with(
            qs_value(json!({ "a": ["b", "c", "d"] })),
            |opts| opts.indices = Some(false),
            "a=b&a=c&a=d",
        );
    }

    // original: omits object key/value pair when value is empty array
    #[test]
    fn omits_object_key_value_pair_when_value_is_empty_array() {
        expect_ok(qs_value(json!({ "a": [], "b": "zz" })), "b=zz");
    }

    // original: should not omit object key/value pair when value is empty array and when asked
    #[test]
    fn should_not_omit_object_key_value_pair_when_value_is_empty_array_and_when_asked() {
        let value = qs_value(json!({ "a": [], "b": "zz" }));

        expect_ok(value.clone(), "b=zz");
        expect_with(
            value.clone(),
            |opts| opts.allow_empty_arrays = false,
            "b=zz",
        );
        expect_with(value, |opts| opts.allow_empty_arrays = true, "a[]&b=zz");
    }

    // original: should throw when allowEmptyArrays is not of type boolean
    #[test]
    fn should_throw_when_allowemptyarrays_is_not_of_type_boolean() {
        let value = qs_value(json!({ "a": [], "b": "zz" }));
        let invalid_values = vec![
            QsValue::String("foobar".to_string()),
            QsValue::Number(0.0),
            QsValue::Number(f64::NAN),
            QsValue::Null,
        ];

        for invalid in invalid_values {
            expect_err(value.clone(), |opts| {
                opts.additional
                    .insert("allowEmptyArrays".to_string(), invalid.clone());
            });
        }
    }

    // original: allowEmptyArrays + strictNullHandling
    #[test]
    fn allowemptyarrays_strictnullhandling() {
        expect_with(
            qs_value(json!({ "testEmptyArray": [] })),
            |opts| {
                opts.strict_null_handling = true;
                opts.allow_empty_arrays = true;
            },
            "testEmptyArray[]",
        );
    }

    // original: stringifies an array value with one item vs multiple items
    #[test]
    fn stringifies_an_array_value_with_one_item_vs_multiple_items() {
        // non-array item
        expect_with(
            qs_value(json!({ "a": "c" })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a=c",
        );

        expect_with(
            qs_value(json!({ "a": "c" })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a=c",
        );

        expect_with(
            qs_value(json!({ "a": "c" })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c",
        );

        expect_with(
            qs_value(json!({ "a": "c" })),
            |opts| {
                opts.encode_values_only = true;
            },
            "a=c",
        );

        // array with a single item
        expect_with(
            qs_value(json!({ "a": ["c"] })),
            |opts| {
                opts.encode_values_only = true;
            },
            "a[0]=c",
        );

        expect_with(
            qs_value(json!({ "a": ["c"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=c",
        );

        expect_with(
            qs_value(json!({ "a": ["c"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c",
        );

        expect_with(
            qs_value(json!({ "a": ["c"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a[]=c",
        );

        expect_with(
            qs_value(json!({ "a": ["c"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0]=c",
        );

        // array with multiple items
        expect_with(
            qs_value(json!({ "a": ["c", "d"] })),
            |opts| {
                opts.encode_values_only = true;
            },
            "a[0]=c&a[1]=d",
        );

        expect_with(
            qs_value(json!({ "a": ["c", "d"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=c&a[]=d",
        );

        expect_with(
            qs_value(json!({ "a": ["c", "d"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c,d",
        );

        expect_with(
            qs_value(json!({ "a": ["c", "d"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a=c,d",
        );

        expect_with(
            qs_value(json!({ "a": ["c", "d"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0]=c&a[1]=d",
        );

        // array with multiple items containing a comma
        expect_with(
            qs_value(json!({ "a": ["c,d", "e"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c%2Cd,e",
        );

        expect_with(
            qs_value(json!({ "a": ["c,d", "e"] })),
            |opts| {
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c%2Cd%2Ce",
        );

        expect_with(
            qs_value(json!({ "a": ["c,d", "e"] })),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a=c%2Cd,e",
        );

        expect_with(
            qs_value(json!({ "a": ["c,d", "e"] })),
            |opts| {
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a=c%2Cd%2Ce",
        );
    }

    // original: non-array item
    #[test]
    fn non_array_item() {
        let value = qs_value(json!({ "a": "c" }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a=c",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a=c",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c",
        );
        expect_with(value, |opts| opts.encode_values_only = true, "a=c");
    }

    // original: array with a single item
    #[test]
    fn array_with_a_single_item() {
        let value = qs_value(json!({ "a": ["c"] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0]=c",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=c",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a[]=c",
        );
        expect_with(value, |opts| opts.encode_values_only = true, "a[0]=c");
    }

    // original: array with multiple items
    #[test]
    fn array_with_multiple_items() {
        let value = qs_value(json!({ "a": ["c", "d"] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0]=c&a[1]=d",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=c&a[]=d",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c,d",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a=c,d",
        );
        expect_with(
            value,
            |opts| opts.encode_values_only = true,
            "a[0]=c&a[1]=d",
        );
    }

    // original: array with multiple items with a comma inside
    #[test]
    fn array_with_multiple_items_with_a_comma_inside() {
        let value = qs_value(json!({ "a": ["c,d", "e"] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c%2Cd,e",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Comma;
            },
            "a=c%2Cd%2Ce",
        );
        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a=c%2Cd,e",
        );
        expect_with(
            value,
            |opts| {
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a=c%2Cd%2Ce",
        );
    }

    // original: stringifies a nested array value
    #[test]
    fn stringifies_a_nested_array_value() {
        let value = qs_value(json!({ "a": { "b": ["c", "d"] } }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[b][0]=c&a[b][1]=d",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[b][]=c&a[b][]=d",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a[b]=c,d",
        );

        expect_with(
            value,
            |opts| opts.encode_values_only = true,
            "a[b][0]=c&a[b][1]=d",
        );
    }

    // original: stringifies comma and empty array values
    #[test]
    fn stringifies_comma_and_empty_array_values() {
        let value = qs_value(json!({ "a": [",", "", "c,d%"] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0]=,&a[1]=&a[2]=c,d%",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=,&a[]=&a[]=c,d%",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=,,,c,d%",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a=,&a=&a=c,d%",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = true;
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0]=%2C&a[1]=&a[2]=c%2Cd%25",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = true;
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=%2C&a[]=&a[]=c%2Cd%25",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = true;
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=%2C,,c%2Cd%25",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = true;
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a=%2C&a=&a=c%2Cd%25",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a%5B0%5D=%2C&a%5B1%5D=&a%5B2%5D=c%2Cd%25",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a%5B%5D=%2C&a%5B%5D=&a%5B%5D=c%2Cd%25",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a=%2C%2C%2Cc%2Cd%25",
        );

        expect_with(
            value,
            |opts| {
                opts.encode = true;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a=%2C&a=&a=c%2Cd%25",
        );
    }

    // original: stringifies comma and empty non-array values
    #[test]
    fn stringifies_comma_and_empty_non_array_values() {
        let value = qs_value(json!({ "a": ",", "b": "", "c": "c,d%" }));

        for encode in [false, true] {
            for encode_values_only in [false, true] {
                for format in [
                    ArrayFormat::Indices,
                    ArrayFormat::Brackets,
                    ArrayFormat::Comma,
                    ArrayFormat::Repeat,
                ] {
                    expect_with(
                        value.clone(),
                        |opts| {
                            opts.encode = encode;
                            opts.encode_values_only = encode_values_only;
                            opts.array_format = format;
                        },
                        if encode {
                            "a=%2C&b=&c=c%2Cd%25"
                        } else {
                            "a=,&b=&c=c,d%"
                        },
                    );
                }
            }
        }
    }

    // original: stringifies a nested array value with dots notation
    #[test]
    fn stringifies_a_nested_array_value_with_dots_notation() {
        let value = qs_value(json!({ "a": { "b": ["c", "d"] } }));

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a.b[0]=c&a.b[1]=d",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a.b[]=c&a.b[]=d",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Comma;
            },
            "a.b=c,d",
        );

        expect_with(
            value,
            |opts| {
                opts.allow_dots = true;
                opts.encode_values_only = true;
            },
            "a.b[0]=c&a.b[1]=d",
        );
    }

    // original: stringifies an object inside an array
    #[test]
    fn stringifies_an_object_inside_an_array() {
        let value = qs_value(json!({ "a": [{ "b": "c" }] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Indices;
                opts.encode_values_only = true;
            },
            "a[0][b]=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Repeat;
                opts.encode_values_only = true;
            },
            "a[b]=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Brackets;
                opts.encode_values_only = true;
            },
            "a[][b]=c",
        );

        expect_with(
            value.clone(),
            |opts| opts.encode_values_only = true,
            "a[0][b]=c",
        );

        let nested = qs_value(json!({ "a": [{ "b": { "c": [1] } }] }));

        expect_with(
            nested.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Indices;
                opts.encode_values_only = true;
            },
            "a[0][b][c][0]=1",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Repeat;
                opts.encode_values_only = true;
            },
            "a[b][c]=1",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Brackets;
                opts.encode_values_only = true;
            },
            "a[][b][c][]=1",
        );

        expect_with(
            nested,
            |opts| opts.encode_values_only = true,
            "a[0][b][c][0]=1",
        );
    }

    // original: stringifies an array with mixed objects and primitives
    #[test]
    fn stringifies_an_array_with_mixed_objects_and_primitives() {
        let value = qs_value(json!({ "a": [{ "b": 1 }, 2, 3] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0][b]=1&a[1]=2&a[2]=3",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[][b]=1&a[]=2&a[]=3",
        );

        expect_with(
            value,
            |opts| opts.encode_values_only = true,
            "a[0][b]=1&a[1]=2&a[2]=3",
        );
    }

    // original: stringifies an object inside an array with dots notation
    #[test]
    fn stringifies_an_object_inside_an_array_with_dots_notation() {
        let value = qs_value(json!({ "a": [{ "b": "c" }] }));

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0].b=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[].b=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode = false;
            },
            "a[0].b=c",
        );

        let nested = qs_value(json!({ "a": [{ "b": { "c": [1] } }] }));

        expect_with(
            nested.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[0].b.c[0]=1",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.allow_dots = true;
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[].b.c[]=1",
        );

        expect_with(
            nested,
            |opts| {
                opts.allow_dots = true;
                opts.encode = false;
            },
            "a[0].b.c[0]=1",
        );
    }

    // original: does not omit object keys when indices = false
    #[test]
    fn does_not_omit_object_keys_when_indices_false() {
        expect_with(
            qs_value(json!({ "a": [{ "b": "c" }] })),
            |opts| opts.indices = Some(false),
            "a%5Bb%5D=c",
        );
    }

    // original: uses indices notation for arrays when indices=true
    #[test]
    fn uses_indices_notation_for_arrays_when_indices_true() {
        expect_with(
            qs_value(json!({ "a": ["b", "c"] })),
            |opts| opts.indices = Some(true),
            "a%5B0%5D=b&a%5B1%5D=c",
        );
    }

    // original: uses indices notation for arrays when no arrayFormat is specified
    #[test]
    fn uses_indices_notation_for_arrays_when_no_arrayformat_is_specified() {
        expect_ok(
            qs_value(json!({ "a": ["b", "c"] })),
            "a%5B0%5D=b&a%5B1%5D=c",
        );
    }

    // original: uses indices notation for arrays when arrayFormat=indices
    #[test]
    fn uses_indices_notation_for_arrays_when_arrayformat_indices() {
        expect_with(
            qs_value(json!({ "a": ["b", "c"] })),
            |opts| opts.array_format = ArrayFormat::Indices,
            "a%5B0%5D=b&a%5B1%5D=c",
        );
    }

    // original: uses repeat notation for arrays when arrayFormat=repeat
    #[test]
    fn uses_repeat_notation_for_arrays_when_arrayformat_repeat() {
        expect_with(
            qs_value(json!({ "a": ["b", "c"] })),
            |opts| opts.array_format = ArrayFormat::Repeat,
            "a=b&a=c",
        );
    }

    // original: uses brackets notation for arrays when arrayFormat=brackets
    #[test]
    fn uses_brackets_notation_for_arrays_when_arrayformat_brackets() {
        expect_with(
            qs_value(json!({ "a": ["b", "c"] })),
            |opts| opts.array_format = ArrayFormat::Brackets,
            "a%5B%5D=b&a%5B%5D=c",
        );
    }

    // original: stringifies a complicated object
    #[test]
    fn stringifies_a_complicated_object() {
        expect_ok(
            qs_value(json!({ "a": { "b": "c", "d": "e" } })),
            "a%5Bb%5D=c&a%5Bd%5D=e",
        );
    }

    // original: stringifies an empty value
    #[test]
    fn stringifies_an_empty_value() {
        expect_ok(qs_value(json!({ "a": "" })), "a=");
        expect_with(
            qs_value(json!({ "a": null })),
            |opts| {
                opts.strict_null_handling = true;
            },
            "a",
        );

        expect_ok(qs_value(json!({ "a": "", "b": "" })), "a=&b=");
        expect_with(
            qs_value(json!({ "a": null, "b": "" })),
            |opts| {
                opts.strict_null_handling = true;
            },
            "a&b=",
        );

        expect_ok(qs_value(json!({ "a": { "b": "" } })), "a%5Bb%5D=");
        expect_with(
            qs_value(json!({ "a": { "b": null } })),
            |opts| {
                opts.strict_null_handling = true;
            },
            "a%5Bb%5D",
        );
    }

    // original: stringifies an empty array in different arrayFormat
    #[test]
    fn stringifies_an_empty_array_in_different_arrayformat() {
        let value = qs_value(json!({ "a": [], "b": [null], "c": "c" }));

        expect_with(value.clone(), |opts| opts.encode = false, "b[0]=&c=c");

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
            },
            "b[0]=&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
            },
            "b[]=&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Repeat;
            },
            "b=&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
            },
            "b=&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "b[]=&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
                opts.strict_null_handling = true;
            },
            "b[0]&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
                opts.strict_null_handling = true;
            },
            "b[]&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Repeat;
                opts.strict_null_handling = true;
            },
            "b&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
                opts.strict_null_handling = true;
            },
            "b&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
                opts.strict_null_handling = true;
                opts.comma_round_trip = true;
            },
            "b[]&c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
                opts.skip_nulls = true;
            },
            "c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
                opts.skip_nulls = true;
            },
            "c=c",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Repeat;
                opts.skip_nulls = true;
            },
            "c=c",
        );

        expect_with(
            value,
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
                opts.skip_nulls = true;
            },
            "c=c",
        );
    }

    // original: stringifies a null object
    #[test]
    fn stringifies_a_null_object() {
        let value = make_object(vec![
            ("__proto__", QsValue::Null),
            ("a", QsValue::String("b".to_string())),
        ]);

        expect_ok(value, "a=b");
    }

    // original: returns an empty string for invalid input
    #[test]
    fn returns_an_empty_string_for_invalid_input() {
        expect_ok(QsValue::Undefined, "");
        expect_ok(QsValue::Bool(false), "");
        expect_ok(QsValue::Null, "");
        expect_ok(QsValue::String(String::new()), "");
    }

    // original: stringifies an object with a null object as a child
    #[test]
    fn stringifies_an_object_with_a_null_object_as_a_child() {
        let child = make_object(vec![
            ("__proto__", QsValue::Null),
            ("b", QsValue::String("c".to_string())),
        ]);
        let value = make_object(vec![("a", child)]);

        expect_ok(value, "a%5Bb%5D=c");
    }

    // original: drops keys with a value of undefined
    #[test]
    fn drops_keys_with_a_value_of_undefined() {
        let only_undefined = make_object(vec![("a", QsValue::Undefined)]);
        expect_ok(only_undefined, "");

        let nested = make_object(vec![(
            "a",
            make_object(vec![("b", QsValue::Undefined), ("c", QsValue::Null)]),
        )]);

        expect_with(
            nested.clone(),
            |opts| {
                opts.strict_null_handling = true;
            },
            "a%5Bc%5D",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.strict_null_handling = false;
            },
            "a%5Bc%5D=",
        );

        let nested_with_empty = make_object(vec![(
            "a",
            make_object(vec![
                ("b", QsValue::Undefined),
                ("c", QsValue::String(String::new())),
            ]),
        )]);

        expect_ok(nested_with_empty, "a%5Bc%5D=");
    }

    // original: url encodes values
    #[test]
    fn url_encodes_values() {
        expect_ok(qs_value(json!({ "a": "b c" })), "a=b%20c");
    }

    // original: stringifies a date
    #[test]
    fn stringifies_a_date() {
        let date = date_from_millis(0);
        expect_ok(
            make_object(vec![("a", date)]),
            "a=1970-01-01T00%3A00%3A00.000Z",
        );
    }

    // original: stringifies the weird object from qs
    #[test]
    fn stringifies_the_weird_object_from_qs() {
        expect_ok(
            make_object(vec![(
                "my weird field",
                QsValue::String("~q1!2\"'w$5&7/z8)?".to_string()),
            )]),
            "my%20weird%20field=~q1%212%22%27w%245%267%2Fz8%29%3F",
        );
    }

    // original: skips properties that are part of the object prototype
    #[test]
    fn skips_properties_that_are_part_of_the_object_prototype() {
        let value = qs_value(json!({ "a": "b" }));
        expect_ok(value.clone(), "a=b");
        expect_ok(qs_value(json!({ "a": { "b": "c" } })), "a%5Bb%5D=c");
    }

    // original: stringifies boolean values
    #[test]
    fn stringifies_boolean_values() {
        expect_ok(qs_value(json!({ "a": true })), "a=true");
        expect_ok(qs_value(json!({ "a": { "b": true } })), "a%5Bb%5D=true");
        expect_ok(qs_value(json!({ "b": false })), "b=false");
        expect_ok(qs_value(json!({ "b": { "c": false } })), "b%5Bc%5D=false");
    }

    // original: stringifies buffer values
    #[test]
    fn stringifies_buffer_values() {
        expect_ok(make_object(vec![("a", bytes(b"test"))]), "a=test");
        expect_ok(
            make_object(vec![("a", make_object(vec![("b", bytes(b"test"))]))]),
            "a%5Bb%5D=test",
        );
    }

    // original: stringifies an object using an alternative delimiter
    #[test]
    fn stringifies_an_object_using_an_alternative_delimiter() {
        expect_with(
            qs_value(json!({ "a": "b", "c": "d" })),
            |opts| opts.delimiter = Some(';'),
            "a=b;c=d",
        );
    }

    // original: does not blow up when Buffer global is missing
    #[test]
    fn does_not_blow_up_when_buffer_global_is_missing() {
        expect_ok(qs_value(json!({ "a": "b", "c": "d" })), "a=b&c=d");
    }

    // original: does not crash when parsing circular references
    #[test]
    fn does_not_crash_when_parsing_circular_references() {
        // mimic an object referencing itself
        let mut circular = make_object(vec![]);
        let circular_clone = circular.clone();
        if let QsValue::Object(ref mut map) = circular {
            map.insert("self".to_string(), circular_clone);
        }
        expect_err_default(circular.clone());

        // mimic an array referencing itself
        let mut circular_array = make_array(vec![]);
        let array_clone = circular_array.clone();
        if let QsValue::Array(ref mut items) = circular_array {
            items.push(array_clone);
        }
        expect_err_default(circular_array);
    }

    // original: non-circular duplicated references can still work
    #[test]
    fn non_circular_duplicated_references_can_still_work() {
        let hour_of_day = make_object(vec![(
            "function",
            QsValue::String("hour_of_day".to_string()),
        )]);

        let p1 = make_object(vec![
            ("function", QsValue::String("gte".to_string())),
            (
                "arguments",
                make_array(vec![hour_of_day.clone(), QsValue::Number(0.0)]),
            ),
        ]);

        let p2 = make_object(vec![
            ("function", QsValue::String("lte".to_string())),
            (
                "arguments",
                make_array(vec![hour_of_day.clone(), QsValue::Number(23.0)]),
            ),
        ]);

        let filters = make_object(vec![("$and", make_array(vec![p1.clone(), p2.clone()]))]);

        let value = make_object(vec![("filters", filters)]);

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "filters[$and][0][function]=gte&filters[$and][0][arguments][0][function]=hour_of_day&filters[$and][0][arguments][1]=0&filters[$and][1][function]=lte&filters[$and][1][arguments][0][function]=hour_of_day&filters[$and][1][arguments][1]=23",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "filters[$and][][function]=gte&filters[$and][][arguments][][function]=hour_of_day&filters[$and][][arguments][]=0&filters[$and][][function]=lte&filters[$and][][arguments][][function]=hour_of_day&filters[$and][][arguments][]=23",
        );

        expect_with(
            value,
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Repeat;
            },
            "filters[$and][function]=gte&filters[$and][arguments][function]=hour_of_day&filters[$and][arguments]=0&filters[$and][function]=lte&filters[$and][arguments][function]=hour_of_day&filters[$and][arguments]=23",
        );
    }

    // original: selects properties when filter=array
    #[test]
    fn selects_properties_when_filter_array() {
        expect_with(
            qs_value(json!({ "a": "b" })),
            |opts| {
                opts.filter = Some(Filter::Keys(vec!["a".to_string()]));
            },
            "a=b",
        );

        expect_with(
            qs_value(json!({ "a": 1 })),
            |opts| {
                opts.filter = Some(Filter::Keys(vec![]));
            },
            "",
        );

        let complex = qs_value(json!({
            "a": { "b": [1, 2, 3, 4], "c": "d" },
            "c": "f"
        }));

        let filter_keys = vec![
            "a".to_string(),
            "b".to_string(),
            "0".to_string(),
            "2".to_string(),
        ];

        expect_with(
            complex.clone(),
            |opts| {
                opts.filter = Some(Filter::Keys(filter_keys.clone()));
                opts.array_format = ArrayFormat::Indices;
            },
            "a%5Bb%5D%5B0%5D=1&a%5Bb%5D%5B2%5D=3",
        );

        expect_with(
            complex.clone(),
            |opts| {
                opts.filter = Some(Filter::Keys(filter_keys.clone()));
                opts.array_format = ArrayFormat::Brackets;
            },
            "a%5Bb%5D%5B%5D=1&a%5Bb%5D%5B%5D=3",
        );

        expect_with(
            complex,
            |opts| {
                opts.filter = Some(Filter::Keys(filter_keys));
            },
            "a%5Bb%5D%5B0%5D=1&a%5Bb%5D%5B2%5D=3",
        );
    }

    // original: supports custom representations when filter=function
    #[test]
    fn supports_custom_representations_when_filter_function() {
        let obj = make_object(vec![
            ("a", QsValue::String("b".to_string())),
            ("c", QsValue::String("d".to_string())),
            (
                "e",
                make_object(vec![("f", date_from_millis(1_257_894_000_000u64))]),
            ),
        ]);

        let call_count = Arc::new(Mutex::new(0usize));
        let original = obj.clone();
        let count_ref = Arc::clone(&call_count);

        let filter = Filter::Function(Arc::new(move |prefix, value| {
            let mut guard = count_ref.lock().expect("lock poisoned");
            *guard += 1;

            if *guard == 1 {
                assert_eq!(prefix, "");
                assert_eq!(value, &original);
                return Some(value.clone());
            }

            if prefix == "c" {
                return None;
            }

            if prefix == "e[f]" {
                #[allow(clippy::collapsible_if)]
                if let QsValue::Date(date) = value {
                    let millis = date
                        .duration_since(UNIX_EPOCH)
                        .expect("date before epoch")
                        .as_millis() as f64;
                    return Some(QsValue::Number(millis));
                }
            }

            Some(value.clone())
        }));

        expect_with(
            obj,
            |opts| {
                opts.filter = Some(filter);
            },
            "a=b&e%5Bf%5D=1257894000000",
        );

        assert_eq!(*call_count.lock().expect("lock poisoned"), 5);
    }

    // original: can disable uri encoding
    #[test]
    fn can_disable_uri_encoding() {
        expect_with(
            qs_value(json!({ "a": "b" })),
            |opts| {
                opts.encode = false;
            },
            "a=b",
        );

        expect_with(
            qs_value(json!({ "a": { "b": "c" } })),
            |opts| {
                opts.encode = false;
            },
            "a[b]=c",
        );

        expect_with(
            qs_value(json!({ "a": "b", "c": null })),
            |opts| {
                opts.encode = false;
                opts.strict_null_handling = true;
            },
            "a=b&c",
        );
    }

    // original: can sort the keys
    #[test]
    fn can_sort_the_keys() {
        let sort = Arc::new(|a: &str, b: &str| a.cmp(b));

        expect_with(
            qs_value(json!({ "a": "c", "z": "y", "b": "f" })),
            |opts| {
                opts.sort = Some(sort.clone());
            },
            "a=c&b=f&z=y",
        );

        expect_with(
            qs_value(json!({ "a": "c", "z": { "j": "a", "i": "b" }, "b": "f" })),
            |opts| {
                opts.sort = Some(sort.clone());
            },
            "a=c&b=f&z%5Bi%5D=b&z%5Bj%5D=a",
        );
    }

    // original: can sort the keys at depth 3 or more too
    #[test]
    fn can_sort_the_keys_at_depth_3_or_more_too() {
        let sort = Arc::new(|a: &str, b: &str| a.cmp(b));

        let value = qs_value(json!({
            "a": "a",
            "z": {
                "zj": { "zjb": "zjb", "zja": "zja" },
                "zi": { "zib": "zib", "zia": "zia" }
            },
            "b": "b"
        }));

        expect_with(
            value.clone(),
            |opts| {
                opts.sort = Some(sort.clone());
                opts.encode = false;
            },
            "a=a&b=b&z[zi][zia]=zia&z[zi][zib]=zib&z[zj][zja]=zja&z[zj][zjb]=zjb",
        );

        expect_with(
            value,
            |opts| {
                opts.sort = None;
                opts.encode = false;
            },
            "a=a&z[zj][zjb]=zjb&z[zj][zja]=zja&z[zi][zib]=zib&z[zi][zia]=zia&b=b",
        );
    }

    // original: can stringify with custom encoding
    #[test]
    fn can_stringify_with_custom_encoding() {
        let value = make_object(vec![
            ("県", QsValue::String("大阪府".to_string())),
            ("", QsValue::String(String::new())),
        ]);

        expect_with(
            value,
            |opts| {
                opts.encoder = Some(Arc::new(
                    |input: &str,
                     default_encoder: &dyn Fn(&str, Charset, ValueKind) -> String,
                     charset: Charset,
                     kind: ValueKind| {
                        if input.is_empty() {
                            return String::new();
                        }

                        match input {
                            "県" => "%8c%a7".to_string(),
                            "大阪府" => "%91%e5%8d%e3%95%7b".to_string(),
                            _ => default_encoder(input, charset, kind),
                        }
                    },
                ));
            },
            "%8c%a7=%91%e5%8d%e3%95%7b&=",
        );
    }

    // original: receives the default encoder as a second argument
    #[test]
    fn receives_the_default_encoder_as_a_second_argument() {
        let captured = Arc::new(Mutex::new(Vec::<(ValueKind, String)>::new()));
        let encoder: EncodeFn = {
            let captured = Arc::clone(&captured);
            Arc::new(
                move |value: &str,
                      default_encoder: &dyn Fn(&str, Charset, ValueKind) -> String,
                      charset: Charset,
                      kind: ValueKind| {
                    let encoded = default_encoder(value, charset, kind);
                    captured
                        .lock()
                        .expect("lock poisoned")
                        .push((kind, value.to_string()));
                    encoded
                },
            )
        };

        let value = make_object(vec![
            ("a", QsValue::Number(1.0)),
            ("b", date_from_millis(0)),
            ("c", QsValue::Bool(true)),
            ("d", make_array(vec![QsValue::Number(1.0)])),
        ]);

        let options = build_stringify_options(|opts| {
            opts.encoder = Some(encoder);
        });

        let _ = stringify_with(&value, options);

        let captured = captured.lock().expect("lock poisoned");
        assert_eq!(
            captured.len(),
            8,
            "encoder should process every key and value"
        );

        let key_values: Vec<String> = captured
            .iter()
            .filter_map(|(kind, value)| {
                if *kind == ValueKind::Key {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect();
        assert!(key_values.contains(&"a".to_string()));
        assert!(key_values.contains(&"b".to_string()));
        assert!(key_values.contains(&"c".to_string()));
        assert!(key_values.contains(&"d[0]".to_string()));

        let value_entries: Vec<String> = captured
            .iter()
            .filter_map(|(kind, value)| {
                if *kind == ValueKind::Value {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect();
        assert!(value_entries.contains(&"1".to_string()));
        assert!(value_entries.contains(&"true".to_string()));
        assert!(value_entries.contains(&"1970-01-01T00:00:00.000Z".to_string()));
    }

    // original: receives the default encoder as a second argument
    #[test]
    fn receives_the_default_encoder_as_a_second_argument_1() {
        let seen = Arc::new(Mutex::new(Vec::<(usize, usize)>::new()));
        let encoder: EncodeFn = {
            let seen = Arc::clone(&seen);
            Arc::new(
                move |value: &str,
                      default_encoder: &dyn Fn(&str, Charset, ValueKind) -> String,
                      charset: Charset,
                      kind: ValueKind| {
                    let raw: (*const (), *const ()) = unsafe { mem::transmute(default_encoder) };
                    seen.lock()
                        .expect("lock poisoned")
                        .push((raw.0 as usize, raw.1 as usize));
                    default_encoder(value, charset, kind)
                },
            )
        };

        let value = make_object(vec![("a", QsValue::Number(1.0))]);
        let options = build_stringify_options(|opts| {
            opts.encoder = Some(encoder);
        });

        let _ = stringify_with(&value, options);

        let seen = seen.lock().expect("lock poisoned");
        assert!(
            !seen.is_empty(),
            "default encoder pointer should be observed"
        );
        let first = seen[0];
        assert!(seen.iter().all(|raw| *raw == first));
    }

    // original: throws error with wrong encoder
    #[test]
    fn throws_error_with_wrong_encoder() {
        expect_err(make_object(vec![]), |opts| {
            opts.additional
                .insert("encoder".to_string(), QsValue::String("string".to_string()));
        });
    }

    // original: can use custom encoder for a buffer object
    #[test]
    fn can_use_custom_encoder_for_a_buffer_object() {
        fn decode_percent(input: &str) -> Option<Vec<u8>> {
            let bytes = input.as_bytes();
            let mut result = Vec::new();
            let mut index = 0;
            while index < bytes.len() {
                if bytes[index] == b'%' {
                    if index + 2 >= bytes.len() {
                        return None;
                    }
                    let hex = input.get(index + 1..index + 3)?;
                    let value = u8::from_str_radix(hex, 16).ok()?;
                    result.push(value);
                    index += 3;
                } else {
                    result.push(bytes[index]);
                    index += 1;
                }
            }
            Some(result)
        }

        fn buffer_encoder() -> EncodeFn {
            Arc::new(
                |value: &str,
                 default_encoder: &dyn Fn(&str, Charset, ValueKind) -> String,
                 charset: Charset,
                 kind: ValueKind| {
                    if matches!(kind, ValueKind::Key) {
                        return default_encoder(value, charset, kind);
                    }

                    if let Some(decoded) = decode_percent(value) {
                        if decoded.len() == 1 {
                            let ch = (decoded[0].saturating_add(b'a')) as char;
                            return ch.to_string();
                        }

                        if let Ok(as_string) = String::from_utf8(decoded) {
                            return as_string;
                        }
                    }

                    default_encoder(value, charset, kind)
                },
            )
        }

        let single = make_object(vec![("a", bytes(&[1]))]);
        expect_with(
            single,
            |opts| {
                opts.encoder = Some(buffer_encoder());
            },
            "a=b",
        );

        let spaced = make_object(vec![("a", bytes(b"a b"))]);
        expect_with(
            spaced,
            |opts| {
                opts.encoder = Some(buffer_encoder());
            },
            "a=a b",
        );
    }

    // original: serializeDate option
    #[test]
    fn serializedate_option() {
        let date = date_from_millis(0);
        expect_ok(
            make_object(vec![("a", date.clone())]),
            "a=1970-01-01T00%3A00%3A00.000Z",
        );

        let mutated_date = date_from_millis(6);
        expect_ok(
            make_object(vec![("a", mutated_date.clone())]),
            "a=1970-01-01T00%3A00%3A00.006Z",
        );

        let specific_date = date_from_millis(6);
        expect_with(
            make_object(vec![("a", specific_date)]),
            |opts| {
                opts.serialize_date = Some(Arc::new(|time| {
                    let millis = time
                        .duration_since(UNIX_EPOCH)
                        .expect("time before epoch")
                        .as_millis();
                    (millis * 7).to_string()
                }));
            },
            "a=42",
        );

        let date_array = make_object(vec![("a", make_array(vec![date.clone()]))]);
        expect_with(
            date_array.clone(),
            |opts| {
                opts.serialize_date = Some(Arc::new(|time| {
                    time.duration_since(UNIX_EPOCH)
                        .expect("time before epoch")
                        .as_millis()
                        .to_string()
                }));
                opts.array_format = ArrayFormat::Comma;
            },
            "a=0",
        );

        expect_with(
            date_array,
            |opts| {
                opts.serialize_date = Some(Arc::new(|time| {
                    time.duration_since(UNIX_EPOCH)
                        .expect("time before epoch")
                        .as_millis()
                        .to_string()
                }));
                opts.array_format = ArrayFormat::Comma;
                opts.comma_round_trip = true;
            },
            "a%5B%5D=0",
        );
    }

    // original: RFC 1738 serialization
    #[test]
    fn rfc_1738_serialization() {
        expect_with(
            qs_value(json!({ "a": "b c" })),
            |opts| opts.format = Format::Rfc1738,
            "a=b+c",
        );

        expect_with(
            qs_value(json!({ "a b": "c d" })),
            |opts| opts.format = Format::Rfc1738,
            "a+b=c+d",
        );

        expect_with(
            make_object(vec![("a b", bytes(b"a b"))]),
            |opts| opts.format = Format::Rfc1738,
            "a+b=a+b",
        );

        expect_with(
            qs_value(json!({ "foo(ref)": "bar" })),
            |opts| opts.format = Format::Rfc1738,
            "foo(ref)=bar",
        );
    }

    // original: RFC 3986 spaces serialization
    #[test]
    fn rfc_3986_spaces_serialization() {
        expect_with(
            qs_value(json!({ "a": "b c" })),
            |opts| opts.format = Format::Rfc3986,
            "a=b%20c",
        );

        expect_with(
            qs_value(json!({ "a b": "c d" })),
            |opts| opts.format = Format::Rfc3986,
            "a%20b=c%20d",
        );

        expect_with(
            make_object(vec![("a b", bytes(b"a b"))]),
            |opts| opts.format = Format::Rfc3986,
            "a%20b=a%20b",
        );
    }

    // original: Backward compatibility to RFC 3986
    #[test]
    fn backward_compatibility_to_rfc_3986() {
        expect_ok(qs_value(json!({ "a": "b c" })), "a=b%20c");
        expect_ok(make_object(vec![("a b", bytes(b"a b"))]), "a%20b=a%20b");
    }

    // original: Edge cases and unknown formats
    #[test]
    fn edge_cases_and_unknown_formats() {
        let invalid_formats = vec![
            QsValue::String("UFO1234".to_string()),
            QsValue::Bool(false),
            QsValue::Number(1234.0),
            QsValue::Null,
            make_object(vec![]),
            make_array(vec![]),
        ];

        for format in invalid_formats {
            expect_err(
                make_object(vec![("a", QsValue::String("b c".to_string()))]),
                |opts| {
                    opts.additional.insert("format".to_string(), format.clone());
                },
            );
        }
    }

    // original: encodeValuesOnly
    #[test]
    fn encodevaluesonly() {
        let value = qs_value(json!({
            "a": "b",
            "c": ["d", "e=f"],
            "f": [["g"], ["h"]]
        }));

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a=b&c[0]=d&c[1]=e%3Df&f[0][0]=g&f[1][0]=h",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a=b&c[]=d&c[]=e%3Df&f[][]=g&f[][]=h",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a=b&c=d&c=e%3Df&f=g&f=h",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Indices;
            },
            "a=b&c%5B0%5D=d&c%5B1%5D=e&f%5B0%5D%5B0%5D=g&f%5B1%5D%5B0%5D=h",
        );

        expect_with(
            value.clone(),
            |opts| {
                opts.array_format = ArrayFormat::Brackets;
            },
            "a=b&c%5B%5D=d&c%5B%5D=e&f%5B%5D%5B%5D=g&f%5B%5D%5B%5D=h",
        );

        expect_with(
            value,
            |opts| {
                opts.array_format = ArrayFormat::Repeat;
            },
            "a=b&c=d&c=e&f=g&f=h",
        );
    }

    // original: encodeValuesOnly - strictNullHandling
    #[test]
    fn encodevaluesonly_strictnullhandling() {
        expect_with(
            qs_value(json!({ "a": { "b": null } })),
            |opts| {
                opts.encode_values_only = true;
                opts.strict_null_handling = true;
            },
            "a[b]",
        );
    }

    // original: throws if an invalid charset is specified
    #[test]
    fn throws_if_an_invalid_charset_is_specified() {
        expect_err(
            make_object(vec![("a", QsValue::String("b".to_string()))]),
            |opts| {
                opts.additional
                    .insert("charset".to_string(), QsValue::String("foobar".to_string()));
            },
        );
    }

    // original: respects a charset of iso-8859-1
    #[test]
    fn respects_a_charset_of_iso_8859_1() {
        expect_with(
            qs_value(json!({ "æ": "æ" })),
            |opts| opts.charset = Charset::Iso88591,
            "%E6=%E6",
        );
    }

    // original: encodes unrepresentable chars as numeric entities in iso-8859-1 mode
    #[test]
    fn encodes_unrepresentable_chars_as_numeric_entities_in_iso_8859_1_mode() {
        expect_with(
            qs_value(json!({ "a": "☺" })),
            |opts| opts.charset = Charset::Iso88591,
            "a=%26%239786%3B",
        );
    }

    // original: respects an explicit charset of utf-8 (the default)
    #[test]
    fn respects_an_explicit_charset_of_utf_8_the_default() {
        expect_with(
            qs_value(json!({ "a": "æ" })),
            |opts| opts.charset = Charset::Utf8,
            "a=%C3%A6",
        );
    }

    // original: `charsetSentinel` option
    #[test]
    fn charsetsentinel_option() {
        expect_with(
            qs_value(json!({ "a": "æ" })),
            |opts| {
                opts.charset = Charset::Utf8;
                opts.charset_sentinel = true;
            },
            "utf8=%E2%9C%93&a=%C3%A6",
        );

        expect_with(
            qs_value(json!({ "a": "æ" })),
            |opts| {
                opts.charset = Charset::Iso88591;
                opts.charset_sentinel = true;
            },
            "utf8=%26%2310003%3B&a=%E6",
        );
    }

    // original: does not mutate the options argument
    #[test]
    fn does_not_mutate_the_options_argument() {
        let mut options = StringifyOptions::default();
        options
            .additional
            .insert("marker".to_string(), QsValue::Number(42.0));
        options.add_query_prefix = true;

        let value = make_object(vec![]);
        let _ = stringify_with(&value, options.clone());

        assert!(options.add_query_prefix);
        assert_eq!(
            options.additional.get("marker"),
            Some(&QsValue::Number(42.0))
        );
    }

    // original: strictNullHandling works with custom filter
    #[test]
    fn strictnullhandling_works_with_custom_filter() {
        let filter = Filter::Function(Arc::new(|_prefix, value| Some(value.clone())));
        let options = build_stringify_options(|opts| {
            opts.strict_null_handling = true;
            opts.filter = Some(filter);
        });

        let result = stringify_with(&make_object(vec![("key", QsValue::Null)]), options)
            .expect("stringify result");
        assert_eq!(result, "key");
    }

    // original: strictNullHandling works with null serializeDate
    #[test]
    fn strictnullhandling_works_with_null_serializedate() {
        let serialize_date = Arc::new(|_date: &SystemTime| String::new());
        let options = build_stringify_options(|opts| {
            opts.strict_null_handling = true;
            opts.serialize_date = Some(serialize_date);
        });

        let date = date_from_millis(0);
        let result =
            stringify_with(&make_object(vec![("key", date)]), options).expect("stringify result");
        assert_eq!(result, "key");
    }

    // original: allows for encoding keys and values differently
    #[test]
    fn allows_for_encoding_keys_and_values_differently() {
        let encoder: EncodeFn = Arc::new(
            |value: &str,
             default_encoder: &dyn Fn(&str, Charset, ValueKind) -> String,
             charset: Charset,
             kind: ValueKind| {
                match kind {
                    ValueKind::Key => default_encoder(value, charset, kind).to_lowercase(),
                    ValueKind::Value => default_encoder(value, charset, kind).to_uppercase(),
                }
            },
        );

        expect_with(
            qs_value(json!({ "KeY": "vAlUe" })),
            |opts| opts.encoder = Some(encoder),
            "key=VALUE",
        );
    }

    // original: objects inside arrays
    #[test]
    fn objects_inside_arrays() {
        let obj = qs_value(json!({ "a": { "b": { "c": "d", "e": "f" } } }));
        let with_array = qs_value(json!({ "a": { "b": [{ "c": "d", "e": "f" }] } }));

        expect_with(
            obj.clone(),
            |opts| opts.encode = false,
            "a[b][c]=d&a[b][e]=f",
        );
        expect_with(
            obj.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[b][c]=d&a[b][e]=f",
        );
        expect_with(
            obj.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[b][c]=d&a[b][e]=f",
        );
        expect_with(
            obj.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a[b][c]=d&a[b][e]=f",
        );
        expect_with(
            obj,
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
            },
            "a[b][c]=d&a[b][e]=f",
        );

        expect_with(
            with_array.clone(),
            |opts| opts.encode = false,
            "a[b][0][c]=d&a[b][0][e]=f",
        );
        expect_with(
            with_array.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[b][][c]=d&a[b][][e]=f",
        );
        expect_with(
            with_array.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[b][0][c]=d&a[b][0][e]=f",
        );
        expect_with(
            with_array.clone(),
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a[b][c]=d&a[b][e]=f",
        );
        expect_with(
            with_array,
            |opts| {
                opts.encode = false;
                opts.array_format = ArrayFormat::Comma;
            },
            "a[b][c]=d&a[b][e]=f",
        );
    }

    // original: stringifies sparse arrays
    #[test]
    fn stringifies_sparse_arrays() {
        let sparse = make_object(vec![(
            "a",
            make_array(vec![
                QsValue::Undefined,
                QsValue::String("2".to_string()),
                QsValue::Undefined,
                QsValue::Undefined,
                QsValue::String("1".to_string()),
            ]),
        )]);

        expect_with(
            sparse.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[1]=2&a[4]=1",
        );

        expect_with(
            sparse.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[]=2&a[]=1",
        );

        expect_with(
            sparse.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a=2&a=1",
        );

        let nested = make_object(vec![(
            "a",
            make_array(vec![
                QsValue::Undefined,
                make_object(vec![(
                    "b",
                    make_array(vec![
                        QsValue::Undefined,
                        QsValue::Undefined,
                        make_object(vec![("c", QsValue::String("1".to_string()))]),
                    ]),
                )]),
            ]),
        )]);

        expect_with(
            nested.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[1][b][2][c]=1",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[][b][][c]=1",
        );

        expect_with(
            nested.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a[b][c]=1",
        );

        let deeper = make_object(vec![(
            "a",
            make_array(vec![
                QsValue::Undefined,
                make_array(vec![
                    QsValue::Undefined,
                    make_array(vec![
                        QsValue::Undefined,
                        QsValue::Undefined,
                        QsValue::Undefined,
                        make_object(vec![(
                            "c",
                            make_array(vec![QsValue::Undefined, QsValue::String("1".to_string())]),
                        )]),
                    ]),
                ]),
            ]),
        )]);

        expect_with(
            deeper.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Indices;
            },
            "a[1][2][3][c][1]=1",
        );

        expect_with(
            deeper.clone(),
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Brackets;
            },
            "a[][][][c][]=1",
        );

        expect_with(
            deeper,
            |opts| {
                opts.encode_values_only = true;
                opts.array_format = ArrayFormat::Repeat;
            },
            "a[c]=1",
        );
    }

    // original: encodes a very long string
    #[test]
    fn encodes_a_very_long_string() {
        let mut chars = Vec::new();
        let mut expected = Vec::new();

        for i in 0..5000 {
            chars.push(format!(" {}", i));
            expected.push(format!("%20{}", i));
        }

        let obj = make_object(vec![("foo", QsValue::String(chars.join("")))]);

        expect_with(
            obj,
            |opts| {
                opts.array_format = ArrayFormat::Brackets;
                opts.charset = Charset::Utf8;
            },
            &format!("foo={}", expected.join("")),
        );
    }
}

// original: stringifies empty keys
mod stringifies_empty_keys {
    use super::*;

    #[derive(serde::Deserialize)]
    struct EmptyTestCase {
        #[serde(rename = "input")]
        _input: String,
        #[serde(rename = "withEmptyKeys")]
        with_empty_keys: serde_json::Value,
        #[serde(rename = "stringifyOutput")]
        stringify_output: CaseOutput,
        #[serde(rename = "noEmptyKeys")]
        _no_empty_keys: Option<serde_json::Value>,
    }

    #[derive(serde::Deserialize)]
    struct CaseOutput {
        indices: String,
        brackets: String,
        repeat: String,
    }

    #[derive(serde::Deserialize)]
    struct EmptyCasesWrapper {
        #[serde(rename = "emptyTestCases")]
        cases: Vec<EmptyTestCase>,
    }

    // original: stringifies an object with empty string key with
    #[test]
    fn stringifies_an_object_with_empty_string_key_with() {
        let data: EmptyCasesWrapper = serde_json::from_str(include_str!("empty-keys-cases.json"))
            .expect("parse empty keys cases");

        for case in data.cases {
            let value = from_json(case.with_empty_keys.clone());

            expect_with(
                value.clone(),
                |opts| {
                    opts.encode = false;
                    opts.array_format = ArrayFormat::Indices;
                },
                &case.stringify_output.indices,
            );

            expect_with(
                value.clone(),
                |opts| {
                    opts.encode = false;
                    opts.array_format = ArrayFormat::Brackets;
                },
                &case.stringify_output.brackets,
            );

            expect_with(
                value.clone(),
                |opts| {
                    opts.encode = false;
                    opts.array_format = ArrayFormat::Repeat;
                },
                &case.stringify_output.repeat,
            );
        }
    }

    // original: edge case with object/arrays
    #[test]
    fn edge_case_with_object_arrays() {
        let first = make_object(vec![(
            "",
            make_object(vec![(
                "",
                make_array(vec![QsValue::Number(2.0), QsValue::Number(3.0)]),
            )]),
        )]);

        expect_with(first.clone(), |opts| opts.encode = false, "[][0]=2&[][1]=3");

        let with_extra = make_object(vec![(
            "",
            make_object(vec![
                (
                    "",
                    make_array(vec![QsValue::Number(2.0), QsValue::Number(3.0)]),
                ),
                ("a", QsValue::Number(2.0)),
            ]),
        )]);

        expect_with(
            with_extra,
            |opts| opts.encode = false,
            "[][0]=2&[][1]=3&[a]=2",
        );
    }

    // original: stringifies non-string keys
    #[test]
    fn stringifies_non_string_keys() {
        let value = make_object(vec![
            ("a", QsValue::String("b".to_string())),
            ("false", make_object(vec![])),
            ("1e+22", QsValue::String("c".to_string())),
            ("d", QsValue::String("e".to_string())),
        ]);

        let filter_keys = vec![
            "a".to_string(),
            "false".to_string(),
            "null".to_string(),
            "10000000000000000000000".to_string(),
            "d".to_string(),
        ];

        expect_with(
            value,
            |opts| {
                opts.filter = Some(Filter::Keys(filter_keys));
                opts.allow_dots = true;
                opts.encode_dot_in_keys = true;
            },
            "a=b&1e%2B22=c&d=e",
        );
    }
}
