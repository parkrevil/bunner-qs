//! Auto-generated skeleton from qs/test/parse.js
#![allow(unused)]

mod common;

use common::*;
use serde_json::json;

// original: parse()
mod parse {
    use super::{
        assert_parse, assert_parse_default, build_options, bytes, from_json, js_date, json,
        make_array, make_object, parse_default, parse_with,
    };
    use bunner_qs::{
        Charset, Delimiter, DepthSetting, DuplicateStrategy, LimitSetting, ParseOptions, QsValue,
        ValueKind,
    };

    // original: parses a simple string
    #[test]
    fn parses_a_simple_string() {
        assert_parse_default(
            "0=foo",
            from_json(json!({
                "0": "foo"
            })),
        );

        assert_parse_default(
            "foo=c++",
            from_json(json!({
                "foo": "c  "
            })),
        );

        assert_parse_default(
            "a[>=]=23",
            from_json(json!({
                "a": { ">=": "23" }
            })),
        );

        assert_parse_default(
            "a[<=>]==23",
            from_json(json!({
                "a": { "<=>": "=23" }
            })),
        );

        assert_parse_default(
            "a[==]=23",
            from_json(json!({
                "a": { "==": "23" }
            })),
        );

        assert_parse(
            "foo",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
            })),
            from_json(json!({ "foo": null })),
        );

        assert_parse_default("foo", from_json(json!({ "foo": "" })));
        assert_parse_default("foo=", from_json(json!({ "foo": "" })));
        assert_parse_default("foo=bar", from_json(json!({ "foo": "bar" })));
        assert_parse_default(
            " foo = bar = baz ",
            from_json(json!({ " foo ": " bar = baz " })),
        );
        assert_parse_default("foo=bar=baz", from_json(json!({ "foo": "bar=baz" })));
        assert_parse_default(
            "foo=bar&bar=baz",
            from_json(json!({ "foo": "bar", "bar": "baz" })),
        );
        assert_parse_default(
            "foo2=bar2&baz2=",
            from_json(json!({ "foo2": "bar2", "baz2": "" })),
        );

        assert_parse(
            "foo=bar&baz",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
            })),
            from_json(json!({
                "foo": "bar",
                "baz": null
            })),
        );

        assert_parse_default(
            "foo=bar&baz",
            from_json(json!({
                "foo": "bar",
                "baz": ""
            })),
        );

        assert_parse_default(
            "cht=p3&chd=t:60,40&chs=250x100&chl=Hello|World",
            from_json(json!({
                "cht": "p3",
                "chd": "t:60,40",
                "chs": "250x100",
                "chl": "Hello|World"
            })),
        );
    }

    // original: comma: false
    #[test]
    fn comma_false() {
        assert_parse_default("a[]=b&a[]=c", from_json(json!({ "a": ["b", "c"] })));
        assert_parse_default("a[0]=b&a[1]=c", from_json(json!({ "a": ["b", "c"] })));
        assert_parse_default("a=b,c", from_json(json!({ "a": "b,c" })));
        assert_parse_default("a=b&a=c", from_json(json!({ "a": ["b", "c"] })));
    }

    // original: comma: true
    #[test]
    fn comma_true() {
        assert_parse(
            "a[]=b&a[]=c",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
        assert_parse(
            "a[0]=b&a[1]=c",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
        assert_parse(
            "a=b,c",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
        assert_parse(
            "a=b&a=c",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
    }

    // original: allows enabling dot notation
    #[test]
    fn allows_enabling_dot_notation() {
        assert_parse_default("a.b=c", from_json(json!({ "a.b": "c" })));

        assert_parse(
            "a.b=c",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({ "a": { "b": "c" } })),
        );
    }

    // original: decode dot keys correctly
    #[test]
    fn decode_dot_keys_correctly() {
        assert_parse(
            "name%252Eobj.first=John&name%252Eobj.last=Doe",
            Some(build_options(|opts| {
                opts.allow_dots = false;
                opts.decode_dot_in_keys = Some(false);
            })),
            from_json(json!({
                "name%2Eobj.first": "John",
                "name%2Eobj.last": "Doe"
            })),
        );

        assert_parse(
            "name.obj.first=John&name.obj.last=Doe",
            Some(build_options(|opts| {
                opts.allow_dots = true;
                opts.decode_dot_in_keys = Some(false);
            })),
            from_json(json!({
                "name": { "obj": { "first": "John", "last": "Doe" } }
            })),
        );

        assert_parse(
            "name%252Eobj.first=John&name%252Eobj.last=Doe",
            Some(build_options(|opts| {
                opts.allow_dots = true;
                opts.decode_dot_in_keys = Some(false);
            })),
            from_json(json!({
                "name%2Eobj": { "first": "John", "last": "Doe" }
            })),
        );

        assert_parse(
            "name%252Eobj.first=John&name%252Eobj.last=Doe",
            Some(build_options(|opts| {
                opts.allow_dots = true;
                opts.decode_dot_in_keys = Some(true);
            })),
            from_json(json!({
                "name.obj": { "first": "John", "last": "Doe" }
            })),
        );

        assert_parse(
            "name%252Eobj%252Esubobject.first%252Egodly%252Ename=John&name%252Eobj%252Esubobject.last=Doe",
            Some(build_options(|opts| {
                opts.allow_dots = false;
                opts.decode_dot_in_keys = Some(false);
            })),
            from_json(json!({
                "name%2Eobj%2Esubobject.first%2Egodly%2Ename": "John",
                "name%2Eobj%2Esubobject.last": "Doe"
            })),
        );

        assert_parse(
            "name.obj.subobject.first.godly.name=John&name.obj.subobject.last=Doe",
            Some(build_options(|opts| {
                opts.allow_dots = true;
                opts.decode_dot_in_keys = Some(false);
            })),
            from_json(json!({
                "name": {
                    "obj": {
                        "subobject": {
                            "first": { "godly": { "name": "John" } },
                            "last": "Doe"
                        }
                    }
                }
            })),
        );

        assert_parse(
            "name%252Eobj%252Esubobject.first%252Egodly%252Ename=John&name%252Eobj%252Esubobject.last=Doe",
            Some(build_options(|opts| {
                opts.allow_dots = true;
                opts.decode_dot_in_keys = Some(true);
            })),
            from_json(json!({
                "name.obj.subobject": {
                    "first.godly.name": "John",
                    "last": "Doe"
                }
            })),
        );

        assert_parse_default(
            "name%252Eobj.first=John&name%252Eobj.last=Doe",
            from_json(json!({
                "name%2Eobj.first": "John",
                "name%2Eobj.last": "Doe"
            })),
        );
    }

    // original: decodes dot in key of object, and allow enabling dot notation when decodeDotInKeys is set to true and allowDots is undefined
    #[test]
    fn decodes_dot_in_key_of_object_and_allow_enabling_dot_notation_when_decodedotinkeys_is_set_to_true_and_allowdots_is_undefined()
     {
        assert_parse(
            "name%252Eobj%252Esubobject.first%252Egodly%252Ename=John&name%252Eobj%252Esubobject.last=Doe",
            Some(build_options(|opts| {
                opts.decode_dot_in_keys = Some(true);
            })),
            from_json(json!({
                "name.obj.subobject": {
                    "first.godly.name": "John",
                    "last": "Doe"
                }
            })),
        );
    }

    // original: throws when decodeDotInKeys is not of type boolean
    #[test]
    fn throws_when_decodedotinkeys_is_not_of_type_boolean() {
        let invalid_values = [
            QsValue::String("foobar".to_string()),
            QsValue::Number(0.0),
            QsValue::Number(f64::NAN),
            QsValue::Null,
        ];

        for invalid in invalid_values.iter() {
            let result = parse_with(
                "foo[]&bar=baz",
                build_options(|opts| {
                    opts.additional
                        .insert("decodeDotInKeys".to_string(), invalid.clone());
                }),
            );

            assert!(
                result.is_err(),
                "expected error for invalid decodeDotInKeys value {invalid:?}"
            );
        }
    }

    // original: allows empty arrays in obj values
    #[test]
    fn allows_empty_arrays_in_obj_values() {
        assert_parse(
            "foo[]&bar=baz",
            Some(build_options(|opts| {
                opts.allow_empty_arrays = true;
            })),
            from_json(json!({
                "foo": [],
                "bar": "baz"
            })),
        );

        assert_parse(
            "foo[]&bar=baz",
            Some(build_options(|opts| {
                opts.allow_empty_arrays = false;
            })),
            from_json(json!({
                "foo": [""],
                "bar": "baz"
            })),
        );
    }

    // original: throws when allowEmptyArrays is not of type boolean
    #[test]
    fn throws_when_allowemptyarrays_is_not_of_type_boolean() {
        let invalid_values = [
            QsValue::String("foobar".to_string()),
            QsValue::Number(0.0),
            QsValue::Number(f64::NAN),
            QsValue::Null,
        ];

        for invalid in invalid_values.iter() {
            let result = parse_with(
                "foo[]&bar=baz",
                build_options(|opts| {
                    opts.additional
                        .insert("allowEmptyArrays".to_string(), invalid.clone());
                }),
            );

            assert!(
                result.is_err(),
                "expected error for invalid allowEmptyArrays value {invalid:?}"
            );
        }
    }

    // original: allowEmptyArrays + strictNullHandling
    #[test]
    fn allowemptyarrays_strictnullhandling() {
        assert_parse(
            "testEmptyArray[]",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
                opts.allow_empty_arrays = true;
            })),
            from_json(json!({ "testEmptyArray": [] })),
        );
    }

    // original: only parses one level when depth = 1
    #[test]
    fn only_parses_one_level_when_depth_1() {
        assert_parse(
            "a[b][c]=d",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(1);
            })),
            from_json(json!({
                "a": { "b": { "[c]": "d" } }
            })),
        );

        assert_parse(
            "a[b][c][d]=e",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(1);
            })),
            from_json(json!({
                "a": { "b": { "[c][d]": "e" } }
            })),
        );
    }

    // original: uses original key when depth = 0
    #[test]
    fn uses_original_key_when_depth_0() {
        assert_parse(
            "a[0]=b&a[1]=c",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(0);
            })),
            from_json(json!({
                "a[0]": "b",
                "a[1]": "c"
            })),
        );

        assert_parse(
            "a[0][0]=b&a[0][1]=c&a[1]=d&e=2",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(0);
            })),
            from_json(json!({
                "a[0][0]": "b",
                "a[0][1]": "c",
                "a[1]": "d",
                "e": "2"
            })),
        );
    }

    // original: uses original key when depth = false
    #[test]
    fn uses_original_key_when_depth_false() {
        assert_parse(
            "a[0]=b&a[1]=c",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Disabled;
            })),
            from_json(json!({
                "a[0]": "b",
                "a[1]": "c"
            })),
        );

        assert_parse(
            "a[0][0]=b&a[0][1]=c&a[1]=d&e=2",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Disabled;
            })),
            from_json(json!({
                "a[0][0]": "b",
                "a[0][1]": "c",
                "a[1]": "d",
                "e": "2"
            })),
        );
    }

    // original: parses an explicit array
    #[test]
    fn parses_an_explicit_array() {
        assert_parse_default("a[]=b", from_json(json!({ "a": ["b"] })));
        assert_parse_default("a[]=b&a[]=c", from_json(json!({ "a": ["b", "c"] })));
        assert_parse_default(
            "a[]=b&a[]=c&a[]=d",
            from_json(json!({ "a": ["b", "c", "d"] })),
        );
    }

    // original: parses a mix of simple and explicit arrays
    #[test]
    fn parses_a_mix_of_simple_and_explicit_arrays() {
        assert_parse_default("a=b&a[]=c", from_json(json!({ "a": ["b", "c"] })));
        assert_parse_default("a[]=b&a=c", from_json(json!({ "a": ["b", "c"] })));
        assert_parse_default("a[0]=b&a=c", from_json(json!({ "a": ["b", "c"] })));
        assert_parse_default("a=b&a[0]=c", from_json(json!({ "a": ["b", "c"] })));

        assert_parse(
            "a[1]=b&a=c",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
        assert_parse(
            "a[]=b&a=c",
            Some(build_options(|opts| {
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
        assert_parse_default("a[]=b&a=c", from_json(json!({ "a": ["b", "c"] })));

        assert_parse(
            "a=b&a[1]=c",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
        assert_parse(
            "a=b&a[]=c",
            Some(build_options(|opts| {
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": ["b", "c"] })),
        );
        assert_parse_default("a=b&a[]=c", from_json(json!({ "a": ["b", "c"] })));
    }

    // original: parses a nested array
    #[test]
    fn parses_a_nested_array() {
        assert_parse_default(
            "a[b][]=c&a[b][]=d",
            from_json(json!({ "a": { "b": ["c", "d"] } })),
        );
        assert_parse_default("a[>=]=25", from_json(json!({ "a": { ">=": "25" } })));
    }

    // original: allows to specify array indices
    #[test]
    fn allows_to_specify_array_indices() {
        assert_parse_default(
            "a[1]=c&a[0]=b&a[2]=d",
            from_json(json!({ "a": ["b", "c", "d"] })),
        );
        assert_parse_default("a[1]=c&a[0]=b", from_json(json!({ "a": ["b", "c"] })));
        assert_parse(
            "a[1]=c",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": ["c"] })),
        );
        assert_parse(
            "a[1]=c",
            Some(build_options(|opts| {
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": { "1": "c" } })),
        );
        assert_parse_default("a[1]=c", from_json(json!({ "a": ["c"] })));
    }

    // original: limits specific array indices to arrayLimit
    #[test]
    fn limits_specific_array_indices_to_arraylimit() {
        assert_parse(
            "a[20]=a",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": ["a"] })),
        );
        assert_parse(
            "a[21]=a",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": { "21": "a" } })),
        );

        assert_parse_default("a[20]=a", from_json(json!({ "a": ["a"] })));
        assert_parse_default("a[21]=a", from_json(json!({ "a": { "21": "a" } })));
    }

    // original: supports encoded = signs
    #[test]
    fn supports_encoded_signs() {
        assert_parse_default(
            "he%3Dllo=th%3Dere",
            from_json(json!({ "he=llo": "th=ere" })),
        );
    }

    // original: is ok with url encoded strings
    #[test]
    fn is_ok_with_url_encoded_strings() {
        assert_parse_default("a[b%20c]=d", from_json(json!({ "a": { "b c": "d" } })));
        assert_parse_default("a[b]=c%20d", from_json(json!({ "a": { "b": "c d" } })));
    }

    // original: allows brackets in the value
    #[test]
    fn allows_brackets_in_the_value() {
        assert_parse_default(
            "pets=[\"tobi\"]",
            from_json(json!({ "pets": "[\"tobi\"]" })),
        );
        assert_parse_default(
            "operators=[\">=\", \"<=\"]",
            from_json(json!({ "operators": "[\">=\", \"<=\"]" })),
        );
    }

    // original: allows empty values
    #[test]
    fn allows_empty_values() {
        assert_parse_default("", from_json(json!({})));
    }

    // original: transforms arrays to objects
    #[test]
    fn transforms_arrays_to_objects() {
        assert_parse_default(
            "foo[0]=bar&foo[bad]=baz",
            from_json(json!({ "foo": { "0": "bar", "bad": "baz" } })),
        );
        assert_parse_default(
            "foo[bad]=baz&foo[0]=bar",
            from_json(json!({ "foo": { "bad": "baz", "0": "bar" } })),
        );
        assert_parse_default(
            "foo[bad]=baz&foo[]=bar",
            from_json(json!({ "foo": { "bad": "baz", "0": "bar" } })),
        );
        assert_parse_default(
            "foo[]=bar&foo[bad]=baz",
            from_json(json!({ "foo": { "0": "bar", "bad": "baz" } })),
        );
        assert_parse_default(
            "foo[bad]=baz&foo[]=bar&foo[]=foo",
            from_json(json!({ "foo": { "bad": "baz", "0": "bar", "1": "foo" } })),
        );
        assert_parse_default(
            "foo[0][a]=a&foo[0][b]=b&foo[1][a]=aa&foo[1][b]=bb",
            from_json(json!({
                "foo": [
                    { "a": "a", "b": "b" },
                    { "a": "aa", "b": "bb" }
                ]
            })),
        );

        assert_parse(
            "a[]=b&a[t]=u&a[hasOwnProperty]=c",
            Some(build_options(|opts| {
                opts.allow_prototypes = false;
            })),
            from_json(json!({ "a": { "0": "b", "t": "u" } })),
        );
        assert_parse(
            "a[]=b&a[t]=u&a[hasOwnProperty]=c",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({
                "a": { "0": "b", "t": "u", "hasOwnProperty": "c" }
            })),
        );
        assert_parse(
            "a[]=b&a[hasOwnProperty]=c&a[x]=y",
            Some(build_options(|opts| {
                opts.allow_prototypes = false;
            })),
            from_json(json!({ "a": { "0": "b", "x": "y" } })),
        );
        assert_parse(
            "a[]=b&a[hasOwnProperty]=c&a[x]=y",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({
                "a": { "0": "b", "hasOwnProperty": "c", "x": "y" }
            })),
        );
    }

    // original: transforms arrays to objects (dot notation)
    #[test]
    fn transforms_arrays_to_objects_dot_notation() {
        assert_parse(
            "foo[0].baz=bar&fool.bad=baz",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": [{ "baz": "bar" }],
                "fool": { "bad": "baz" }
            })),
        );

        assert_parse(
            "foo[0].baz=bar&fool.bad.boo=baz",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": [{ "baz": "bar" }],
                "fool": { "bad": { "boo": "baz" } }
            })),
        );

        assert_parse(
            "foo[0][0].baz=bar&fool.bad=baz",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": [[{ "baz": "bar" }]],
                "fool": { "bad": "baz" }
            })),
        );

        assert_parse(
            "foo[0].baz[0]=15&foo[0].bar=2",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": [{ "baz": ["15"], "bar": "2" }]
            })),
        );

        assert_parse(
            "foo[0].baz[0]=15&foo[0].baz[1]=16&foo[0].bar=2",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": [{ "baz": ["15", "16"], "bar": "2" }]
            })),
        );

        assert_parse(
            "foo.bad=baz&foo[0]=bar",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": { "bad": "baz", "0": "bar" }
            })),
        );

        assert_parse(
            "foo.bad=baz&foo[]=bar",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": { "bad": "baz", "0": "bar" }
            })),
        );

        assert_parse(
            "foo[]=bar&foo.bad=baz",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": { "0": "bar", "bad": "baz" }
            })),
        );

        assert_parse(
            "foo.bad=baz&foo[]=bar&foo[]=foo",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": { "bad": "baz", "0": "bar", "1": "foo" }
            })),
        );

        assert_parse(
            "foo[0].a=a&foo[0].b=b&foo[1].a=aa&foo[1].b=bb",
            Some(build_options(|opts| {
                opts.allow_dots = true;
            })),
            from_json(json!({
                "foo": [
                    { "a": "a", "b": "b" },
                    { "a": "aa", "b": "bb" }
                ]
            })),
        );
    }

    // original: correctly prunes undefined values when converting an array to an object
    #[test]
    fn correctly_prunes_undefined_values_when_converting_an_array_to_an_object() {
        assert_parse_default(
            "a[2]=b&a[99999999]=c",
            from_json(json!({ "a": { "2": "b", "99999999": "c" } })),
        );
    }

    // original: supports malformed uri characters
    #[test]
    fn supports_malformed_uri_characters() {
        assert_parse(
            "{%:%}",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
            })),
            from_json(json!({ "{%:%}": null })),
        );
        assert_parse_default("{%:%}=", from_json(json!({ "{%:%}": "" })));
        assert_parse_default("foo=%:%}", from_json(json!({ "foo": "%:%}" })));
    }

    // original: doesn't produce empty keys
    #[test]
    fn doesn_t_produce_empty_keys() {
        assert_parse_default("_r=1&", from_json(json!({ "_r": "1" })));
    }

    // original: cannot access Object prototype
    #[test]
    fn cannot_access_object_prototype() {
        assert_parse_default(
            "constructor[prototype][bad]=bad",
            from_json(json!({
                "constructor": { "prototype": { "bad": "bad" } }
            })),
        );
        assert_parse_default(
            "bad[constructor][prototype][bad]=bad",
            from_json(json!({
                "bad": {
                    "constructor": { "prototype": { "bad": "bad" } }
                }
            })),
        );
    }

    // original: parses arrays of objects
    #[test]
    fn parses_arrays_of_objects() {
        assert_parse_default("a[][b]=c", from_json(json!({ "a": [{ "b": "c" }] })));
        assert_parse_default("a[0][b]=c", from_json(json!({ "a": [{ "b": "c" }] })));
    }

    // original: allows for empty strings in arrays
    #[test]
    fn allows_for_empty_strings_in_arrays() {
        assert_parse_default(
            "a[]=b&a[]=&a[]=c",
            from_json(json!({ "a": ["b", "", "c"] })),
        );

        assert_parse(
            "a[0]=b&a[1]&a[2]=c&a[19]=",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": ["b", null, "c", ""] })),
        );

        assert_parse(
            "a[]=b&a[]&a[]=c&a[]=",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": ["b", null, "c", ""] })),
        );

        assert_parse(
            "a[0]=b&a[1]=&a[2]=c&a[19]",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": ["b", "", "c", null] })),
        );

        assert_parse(
            "a[]=b&a[]=&a[]=c&a[]",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": ["b", "", "c", null] })),
        );

        assert_parse_default(
            "a[]=&a[]=b&a[]=c",
            from_json(json!({ "a": ["", "b", "c"] })),
        );
    }

    // original: compacts sparse arrays
    #[test]
    fn compacts_sparse_arrays() {
        assert_parse(
            "a[10]=1&a[2]=2",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({ "a": ["2", "1"] })),
        );
        assert_parse(
            "a[1][b][2][c]=1",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({
                "a": [{ "b": [{ "c": "1" }] }]
            })),
        );
        assert_parse(
            "a[1][2][3][c]=1",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({
                "a": [[[{ "c": "1" }]]]
            })),
        );
        assert_parse(
            "a[1][2][3][c][1]=1",
            Some(build_options(|opts| {
                opts.array_limit = 20;
            })),
            from_json(json!({
                "a": [[[{ "c": ["1"] }]]]
            })),
        );
    }

    // original: parses sparse arrays
    #[test]
    fn parses_sparse_arrays() {
        assert_parse(
            "a[4]=1&a[1]=2",
            Some(build_options(|opts| {
                opts.allow_sparse = true;
            })),
            from_json(json!({
                "a": [null, "2", null, null, "1"]
            })),
        );
        assert_parse(
            "a[1][b][2][c]=1",
            Some(build_options(|opts| {
                opts.allow_sparse = true;
            })),
            from_json(json!({
                "a": [null, { "b": [null, null, { "c": "1" }] }]
            })),
        );
        assert_parse(
            "a[1][2][3][c]=1",
            Some(build_options(|opts| {
                opts.allow_sparse = true;
            })),
            from_json(json!({
                "a": [null, [null, null, [null, null, null, { "c": "1" }]]]
            })),
        );
        assert_parse(
            "a[1][2][3][c][1]=1",
            Some(build_options(|opts| {
                opts.allow_sparse = true;
            })),
            from_json(json!({
                "a": [null, [null, null, [null, null, null, { "c": [null, "1"] }]]]
            })),
        );
    }

    // original: parses semi-parsed strings
    #[test]
    fn parses_semi_parsed_strings() {
        assert_parse_default("a[b]=c", from_json(json!({ "a": { "b": "c" } })));

        assert_parse_default(
            "a[b]=c&a[d]=e",
            from_json(json!({ "a": { "b": "c", "d": "e" } })),
        );
    }

    // original: parses buffers correctly
    #[test]
    fn parses_buffers_correctly() {
        use std::sync::Arc;

        let decoder = Arc::new(
            |input: &str,
             default_decoder: &dyn Fn(&str, Charset, ValueKind) -> QsValue,
             charset: Charset,
             kind: ValueKind| {
                match kind {
                    ValueKind::Key => default_decoder(input, charset, kind),
                    ValueKind::Value => QsValue::Bytes(input.as_bytes().to_vec()),
                }
            },
        );

        assert_parse(
            "a=test",
            Some(build_options(|opts| {
                opts.decoder = Some(decoder.clone());
            })),
            make_object(vec![("a", bytes(b"test"))]),
        );
    }

    // original: parses jquery-param strings
    #[test]
    fn parses_jquery_param_strings() {
        let encoded = "filter%5B0%5D%5B%5D=int1&filter%5B0%5D%5B%5D=%3D&filter%5B0%5D%5B%5D=77&filter%5B%5D=and&filter%5B2%5D%5B%5D=int2&filter%5B2%5D%5B%5D=%3D&filter%5B2%5D%5B%5D=8";
        assert_parse_default(
            encoded,
            from_json(json!({
                "filter": [
                    ["int1", "=", "77"],
                    "and",
                    ["int2", "=", "8"]
                ]
            })),
        );
    }

    // original: continues parsing when no parent is found
    #[test]
    fn continues_parsing_when_no_parent_is_found() {
        assert_parse_default(
            "[]=&a=b",
            from_json(json!({
                "0": "",
                "a": "b"
            })),
        );

        assert_parse(
            "[]&a=b",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
            })),
            from_json(json!({
                "0": null,
                "a": "b"
            })),
        );

        assert_parse_default("[foo]=bar", from_json(json!({ "foo": "bar" })));
    }

    // original: does not error when parsing a very long array
    #[test]
    fn does_not_error_when_parsing_a_very_long_array() {
        let mut query = String::from("a[]=a");
        while query.len() < 128 * 1024 {
            let clone = query.clone();
            query.push('&');
            query.push_str(&clone);
        }

        let result = parse_default(&query);
        assert!(result.is_ok());
    }

    // original: does not throw when a native prototype has an enumerable property
    #[test]
    fn does_not_throw_when_a_native_prototype_has_an_enumerable_property() {
        assert_parse_default("a=b", from_json(json!({ "a": "b" })));
        assert_parse_default("a[][b]=c", from_json(json!({ "a": [{ "b": "c" }] })));
    }

    // original: parses a string with an alternative string delimiter
    #[test]
    fn parses_a_string_with_an_alternative_string_delimiter() {
        assert_parse(
            "a=b;c=d",
            Some(build_options(|opts| {
                opts.delimiter = Delimiter::Char(';');
            })),
            from_json(json!({
                "a": "b",
                "c": "d"
            })),
        );
    }

    // original: parses a string with an alternative RegExp delimiter
    #[test]
    fn parses_a_string_with_an_alternative_regexp_delimiter() {
        assert_parse(
            "a=b; c=d",
            Some(build_options(|opts| {
                opts.delimiter = Delimiter::Regex("[;,] *".to_string());
            })),
            from_json(json!({
                "a": "b",
                "c": "d"
            })),
        );
    }

    // original: does not use non-splittable objects as delimiters
    #[test]
    fn does_not_use_non_splittable_objects_as_delimiters() {
        assert_parse(
            "a=b&c=d",
            Some(build_options(|opts| {
                opts.delimiter = Delimiter::Other;
            })),
            from_json(json!({
                "a": "b",
                "c": "d"
            })),
        );
    }

    // original: allows overriding parameter limit
    #[test]
    fn allows_overriding_parameter_limit() {
        assert_parse(
            "a=b&c=d",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(1);
            })),
            from_json(json!({ "a": "b" })),
        );
    }

    // original: allows setting the parameter limit to Infinity
    #[test]
    fn allows_setting_the_parameter_limit_to_infinity() {
        assert_parse(
            "a=b&c=d",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Infinite;
            })),
            from_json(json!({
                "a": "b",
                "c": "d"
            })),
        );
    }

    // original: allows overriding array limit
    #[test]
    fn allows_overriding_array_limit() {
        assert_parse(
            "a[0]=b",
            Some(build_options(|opts| {
                opts.array_limit = -1;
            })),
            from_json(json!({ "a": { "0": "b" } })),
        );
        assert_parse(
            "a[0]=b",
            Some(build_options(|opts| {
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": ["b"] })),
        );

        assert_parse(
            "a[-1]=b",
            Some(build_options(|opts| {
                opts.array_limit = -1;
            })),
            from_json(json!({ "a": { "-1": "b" } })),
        );
        assert_parse(
            "a[-1]=b",
            Some(build_options(|opts| {
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": { "-1": "b" } })),
        );

        assert_parse(
            "a[0]=b&a[1]=c",
            Some(build_options(|opts| {
                opts.array_limit = -1;
            })),
            from_json(json!({ "a": { "0": "b", "1": "c" } })),
        );
        assert_parse(
            "a[0]=b&a[1]=c",
            Some(build_options(|opts| {
                opts.array_limit = 0;
            })),
            from_json(json!({ "a": { "0": "b", "1": "c" } })),
        );
    }

    // original: allows disabling array parsing
    #[test]
    fn allows_disabling_array_parsing() {
        assert_parse(
            "a[0]=b&a[1]=c",
            Some(build_options(|opts| {
                opts.parse_arrays = false;
            })),
            from_json(json!({ "a": { "0": "b", "1": "c" } })),
        );

        assert_parse(
            "a[]=b",
            Some(build_options(|opts| {
                opts.parse_arrays = false;
            })),
            from_json(json!({ "a": { "0": "b" } })),
        );
    }

    // original: allows for query string prefix
    #[test]
    fn allows_for_query_string_prefix() {
        assert_parse(
            "?foo=bar",
            Some(build_options(|opts| {
                opts.ignore_query_prefix = true;
            })),
            from_json(json!({ "foo": "bar" })),
        );
        assert_parse(
            "foo=bar",
            Some(build_options(|opts| {
                opts.ignore_query_prefix = true;
            })),
            from_json(json!({ "foo": "bar" })),
        );
        assert_parse_default("?foo=bar", from_json(json!({ "?foo": "bar" })));
    }

    // original: parses an object
    #[test]
    fn parses_an_object() {
        assert_parse(
            "user[name][pop%5Bbob%5D]=3&user[email]",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
            })),
            from_json(json!({
                "user": {
                    "name": { "pop[bob]": "3" },
                    "email": null
                }
            })),
        );
    }

    // original: parses string with comma as array divider
    #[test]
    fn parses_string_with_comma_as_array_divider() {
        assert_parse(
            "foo=bar,tee",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": ["bar", "tee"] })),
        );
        assert_parse(
            "foo[bar]=coffee,tee",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": { "bar": ["coffee", "tee"] } })),
        );
        assert_parse(
            "foo=",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": "" })),
        );
        assert_parse(
            "foo",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": "" })),
        );
        assert_parse(
            "foo",
            Some(build_options(|opts| {
                opts.comma = true;
                opts.strict_null_handling = true;
            })),
            from_json(json!({ "foo": null })),
        );

        assert_parse_default("a[0]=c", from_json(json!({ "a": ["c"] })));
        assert_parse_default("a[]=c", from_json(json!({ "a": ["c"] })));
        assert_parse(
            "a[]=c",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "a": ["c"] })),
        );

        assert_parse_default("a[0]=c&a[1]=d", from_json(json!({ "a": ["c", "d"] })));
        assert_parse_default("a[]=c&a[]=d", from_json(json!({ "a": ["c", "d"] })));
        assert_parse(
            "a=c,d",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "a": ["c", "d"] })),
        );
    }

    // original: parses values with comma as array divider
    #[test]
    fn parses_values_with_comma_as_array_divider() {
        assert_parse(
            "foo=bar,tee",
            Some(build_options(|opts| {
                opts.comma = false;
            })),
            from_json(json!({ "foo": "bar,tee" })),
        );

        assert_parse(
            "foo=bar,tee",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": ["bar", "tee"] })),
        );
    }

    // original: use number decoder, parses string that has one number with comma option enabled
    #[test]
    fn use_number_decoder_parses_string_that_has_one_number_with_comma_option_enabled() {
        use std::sync::Arc;

        let decoder = Arc::new(
            |input: &str,
             default_decoder: &dyn Fn(&str, Charset, ValueKind) -> QsValue,
             charset: Charset,
             kind: ValueKind| {
                if kind == ValueKind::Value
                    && let Ok(number) = input.parse::<f64>()
                {
                    return QsValue::Number(number);
                }

                default_decoder(input, charset, kind)
            },
        );

        assert_parse(
            "foo=1",
            Some(build_options(|opts| {
                opts.comma = true;
                opts.decoder = Some(decoder.clone());
            })),
            from_json(json!({ "foo": 1 })),
        );

        assert_parse(
            "foo=0",
            Some(build_options(|opts| {
                opts.comma = true;
                opts.decoder = Some(decoder.clone());
            })),
            from_json(json!({ "foo": 0 })),
        );
    }

    // original: parses brackets holds array of arrays when having two parts of strings with comma as array divider
    #[test]
    fn parses_brackets_holds_array_of_arrays_when_having_two_parts_of_strings_with_comma_as_array_divider()
     {
        assert_parse(
            "foo[]=1,2,3&foo[]=4,5,6",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    ["4", "5", "6"]
                ]
            })),
        );
        assert_parse(
            "foo[]=1,2,3&foo[]=",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    ""
                ]
            })),
        );
        assert_parse(
            "foo[]=1,2,3&foo[]=,",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    ["", ""]
                ]
            })),
        );
        assert_parse(
            "foo[]=1,2,3&foo[]=a",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    "a"
                ]
            })),
        );
    }

    // original: parses url-encoded brackets holds array of arrays when having two parts of strings with comma as array divider
    #[test]
    fn parses_url_encoded_brackets_holds_array_of_arrays_when_having_two_parts_of_strings_with_comma_as_array_divider()
     {
        assert_parse(
            "foo%5B%5D=1,2,3&foo%5B%5D=4,5,6",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    ["4", "5", "6"]
                ]
            })),
        );
        assert_parse(
            "foo%5B%5D=1,2,3&foo%5B%5D=",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    ""
                ]
            })),
        );
        assert_parse(
            "foo%5B%5D=1,2,3&foo%5B%5D=,",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    ["", ""]
                ]
            })),
        );
        assert_parse(
            "foo%5B%5D=1,2,3&foo%5B%5D=a",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({
                "foo": [
                    ["1", "2", "3"],
                    "a"
                ]
            })),
        );
    }

    // original: parses comma delimited array while having percent-encoded comma treated as normal text
    #[test]
    fn parses_comma_delimited_array_while_having_percent_encoded_comma_treated_as_normal_text() {
        assert_parse(
            "foo=a%2Cb",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": "a,b" })),
        );
        assert_parse(
            "foo=a%2C%20b,d",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": ["a, b", "d"] })),
        );
        assert_parse(
            "foo=a%2C%20b,c%2C%20d",
            Some(build_options(|opts| {
                opts.comma = true;
            })),
            from_json(json!({ "foo": ["a, b", "c, d"] })),
        );
    }

    // original: parses an object in dot notation
    #[test]
    fn parses_an_object_in_dot_notation() {
        assert_parse(
            "user.name[pop%5Bbob%5D]=3&user.email",
            Some(build_options(|opts| {
                opts.allow_dots = true;
                opts.strict_null_handling = true;
            })),
            from_json(json!({
                "user": {
                    "name": { "pop[bob]": "3" },
                    "email": null
                }
            })),
        );
    }

    // original: parses an object and not child values
    #[test]
    fn parses_an_object_and_not_child_values() {
        assert_parse(
            "user[name][pop%5Bbob%5D][test]=3&user[email]",
            Some(build_options(|opts| {
                opts.strict_null_handling = true;
            })),
            from_json(json!({
                "user": {
                    "name": {
                        "pop[bob]": { "test": "3" }
                    },
                    "email": null
                }
            })),
        );
    }

    // original: does not blow up when Buffer global is missing
    #[test]
    fn does_not_blow_up_when_buffer_global_is_missing() {
        assert_parse_default(
            "a=b&c=d",
            from_json(json!({
                "a": "b",
                "c": "d"
            })),
        );
    }

    // original: does not crash when parsing circular references
    #[test]
    fn does_not_crash_when_parsing_circular_references() {
        assert_parse_default(
            "foo[bar]=baz&foo[baz]=qux",
            from_json(json!({
                "foo": {
                    "bar": "baz",
                    "baz": "qux"
                }
            })),
        );
    }

    // original: does not crash when parsing deep objects
    #[test]
    fn does_not_crash_when_parsing_deep_objects() {
        let mut query = String::from("foo");
        for _ in 0..5000 {
            query.push_str("[p]");
        }
        query.push_str("=bar");

        let result = parse_with(
            &query,
            build_options(|opts| {
                opts.depth = DepthSetting::Finite(5000);
            }),
        )
        .expect("expected deep object parse to succeed");

        let mut current = match &result {
            QsValue::Object(map) => map.get("foo").expect("expected root object to contain foo"),
            _ => panic!("expected root to be an object"),
        };

        let mut depth = 0usize;
        while let QsValue::Object(map) = current {
            if let Some(next) = map.get("p") {
                depth += 1;
                current = next;
            } else {
                break;
            }
        }

        assert_eq!(depth, 5000);

        match current {
            QsValue::String(value) => assert_eq!(value, "bar"),
            _ => panic!("expected deepest value to be the string 'bar'"),
        }
    }

    // original: parses null objects correctly
    #[test]
    fn parses_null_objects_correctly() {
        assert_parse(
            "b=c",
            Some(build_options(|opts| {
                opts.plain_objects = true;
            })),
            from_json(json!({
                "__proto__": null,
                "b": "c"
            })),
        );

        let result = parse_with(
            "a[b]=c",
            build_options(|opts| {
                opts.plain_objects = true;
            }),
        )
        .expect("expected parse to succeed with plainObjects");

        match result {
            QsValue::Object(root) => {
                assert!(root.contains_key("a"), "result has \"a\" property");
                match root.get("a").expect("missing a property") {
                    QsValue::Object(inner) => {
                        assert_eq!(inner.get("__proto__"), Some(&QsValue::Null));
                        assert_eq!(inner.get("b"), Some(&QsValue::String("c".to_string())));
                    }
                    other => panic!("expected plain object, got {other:?}"),
                }
            }
            other => panic!("expected root object, got {other:?}"),
        }
    }

    // original: parses dates correctly
    #[test]
    fn parses_dates_correctly() {
        use std::sync::Arc;
        use std::time::{Duration, SystemTime};

        let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
        let decoder = Arc::new(
            move |input: &str,
                  default_decoder: &dyn Fn(&str, Charset, ValueKind) -> QsValue,
                  charset: Charset,
                  kind: ValueKind| {
                if kind == ValueKind::Value && input == "now" {
                    return QsValue::Date(timestamp);
                }

                default_decoder(input, charset, kind)
            },
        );

        assert_parse(
            "a=now",
            Some(build_options(|opts| {
                opts.decoder = Some(decoder.clone());
            })),
            make_object(vec![("a", js_date(timestamp))]),
        );
    }

    // original: parses regular expressions correctly
    #[test]
    fn parses_regular_expressions_correctly() {
        use std::sync::Arc;

        let decoder = Arc::new(
            |input: &str,
             default_decoder: &dyn Fn(&str, Charset, ValueKind) -> QsValue,
             charset: Charset,
             kind: ValueKind| {
                if kind == ValueKind::Value {
                    return QsValue::Regex(input.to_string());
                }

                default_decoder(input, charset, kind)
            },
        );

        assert_parse(
            "a=%5Etest%24",
            Some(build_options(|opts| {
                opts.decoder = Some(decoder.clone());
            })),
            make_object(vec![("a", QsValue::Regex("^test$".to_string()))]),
        );
    }

    // original: does not allow overwriting prototype properties
    #[test]
    fn does_not_allow_overwriting_prototype_properties() {
        assert_parse(
            "a[hasOwnProperty]=b",
            Some(build_options(|opts| {
                opts.allow_prototypes = false;
            })),
            from_json(json!({})),
        );

        assert_parse(
            "hasOwnProperty=b",
            Some(build_options(|opts| {
                opts.allow_prototypes = false;
            })),
            from_json(json!({})),
        );

        assert_parse(
            "toString",
            Some(build_options(|opts| {
                opts.allow_prototypes = false;
            })),
            from_json(json!({})),
        );
    }

    // original: can allow overwriting prototype properties
    #[test]
    fn can_allow_overwriting_prototype_properties() {
        assert_parse(
            "a[hasOwnProperty]=b",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({ "a": { "hasOwnProperty": "b" } })),
        );

        assert_parse(
            "hasOwnProperty=b",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({ "hasOwnProperty": "b" })),
        );

        assert_parse(
            "toString",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({ "toString": "" })),
        );

        assert_parse(
            "a[b]=c&a=toString",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
                opts.plain_objects = true;
            })),
            from_json(json!({
                "__proto__": null,
                "a": {
                    "__proto__": null,
                    "b": "c",
                    "toString": true
                }
            })),
        );
    }

    // original: does not crash when the global Object prototype is frozen
    #[test]
    fn does_not_crash_when_the_global_object_prototype_is_frozen() {
        assert_parse(
            "frozenProp",
            Some(build_options(|opts| {
                opts.allow_prototypes = false;
            })),
            from_json(json!({})),
        );
    }

    // original: params starting with a closing bracket
    #[test]
    fn params_starting_with_a_closing_bracket() {
        assert_parse_default("]]=toString", from_json(json!({ "]]": "toString" })));
        assert_parse_default("]]=toString", from_json(json!({ "]]": "toString" })));
        assert_parse_default(
            "]hello]=toString",
            from_json(json!({ "]hello]": "toString" })),
        );
    }

    // original: params starting with a starting bracket
    #[test]
    fn params_starting_with_a_starting_bracket() {
        assert_parse_default("[=toString", from_json(json!({ "[": "toString" })));
        assert_parse_default("[[=toString", from_json(json!({ "[[": "toString" })));
        assert_parse_default(
            "[hello[=toString",
            from_json(json!({ "[hello[": "toString" })),
        );
    }

    // original: add keys to objects
    #[test]
    fn add_keys_to_objects() {
        assert_parse_default(
            "a[b]=c&a=d",
            from_json(json!({ "a": { "b": "c", "d": true } })),
        );

        assert_parse_default("a[b]=c&a=toString", from_json(json!({ "a": { "b": "c" } })));

        assert_parse(
            "a[b]=c&a=toString",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({ "a": { "b": "c", "toString": true } })),
        );
    }

    // original: dunder proto is ignored
    #[test]
    fn dunder_proto_is_ignored() {
        assert_parse(
            "categories[__proto__]=login&categories[__proto__]&categories[length]=42",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({
                "categories": { "length": "42" }
            })),
        );

        assert_parse(
            "categories[__proto__]=login&categories[__proto__]&categories[length]=42",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
                opts.plain_objects = true;
            })),
            from_json(json!({
                "__proto__": null,
                "categories": {
                    "__proto__": null,
                    "length": "42"
                }
            })),
        );

        assert_parse(
            "categories[__proto__]=cats&categories[__proto__]=dogs&categories[some][json]=toInject",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({
                "categories": {
                    "some": { "json": "toInject" }
                }
            })),
        );

        assert_parse(
            "foo[__proto__][hidden]=value&foo[bar]=stuffs",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
            })),
            from_json(json!({
                "foo": {
                    "bar": "stuffs"
                }
            })),
        );

        assert_parse(
            "foo[__proto__][hidden]=value&foo[bar]=stuffs",
            Some(build_options(|opts| {
                opts.allow_prototypes = true;
                opts.plain_objects = true;
            })),
            from_json(json!({
                "__proto__": null,
                "foo": {
                    "__proto__": null,
                    "bar": "stuffs"
                }
            })),
        );
    }

    // original: can return null objects
    #[test]
    fn can_return_null_objects() {
        assert_parse(
            "a[b]=c&a[hasOwnProperty]=d",
            Some(build_options(|opts| {
                opts.plain_objects = true;
            })),
            from_json(json!({
                "__proto__": null,
                "a": {
                    "__proto__": null,
                    "b": "c",
                    "hasOwnProperty": "d"
                }
            })),
        );

        assert_parse(
            "",
            Some(build_options(|opts| {
                opts.plain_objects = true;
            })),
            from_json(json!({
                "__proto__": null
            })),
        );

        assert_parse(
            "a[]=b&a[c]=d",
            Some(build_options(|opts| {
                opts.plain_objects = true;
            })),
            from_json(json!({
                "__proto__": null,
                "a": {
                    "__proto__": null,
                    "0": "b",
                    "c": "d"
                }
            })),
        );
    }

    // original: can parse with custom encoding
    #[test]
    fn can_parse_with_custom_encoding() {
        let shift_jis_encoded = "%8c%a7=%91%e5%8d%e3%95%7b";

        assert_parse(
            shift_jis_encoded,
            Some(build_options(|opts| {
                opts.decoder = Some(std::sync::Arc::new(
                    |input, default_decoder, charset, kind| default_decoder(input, charset, kind),
                ));
            })),
            from_json(json!({ "県": "大阪府" })),
        );
    }

    // original: receives the default decoder as a second argument
    #[test]
    fn receives_the_default_decoder_as_a_second_argument() {
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag_clone = flag.clone();

        let _ = parse_with(
            "a",
            build_options(|opts| {
                opts.decoder = Some(std::sync::Arc::new(
                    move |_input, default_decoder, charset, kind| {
                        flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                        default_decoder("", charset, kind)
                    },
                ));
            }),
        );

        assert!(flag.load(std::sync::atomic::Ordering::SeqCst));
    }

    // original: throws error with wrong decoder
    #[test]
    fn throws_error_with_wrong_decoder() {
        let result = parse_with(
            "",
            build_options(|opts| {
                opts.additional
                    .insert("decoder".to_string(), QsValue::String("string".to_string()));
            }),
        );

        assert!(result.is_err());
    }

    // original: does not mutate the options argument
    #[test]
    fn does_not_mutate_the_options_argument() {
        let mut options = ParseOptions::default();
        options
            .additional
            .insert("sentinel".to_string(), QsValue::String("value".to_string()));

        let _ = parse_with("a[b]=true", options.clone());

        assert_eq!(
            options.additional.get("sentinel"),
            Some(&QsValue::String("value".to_string()))
        );
    }

    // original: throws if an invalid charset is specified
    #[test]
    fn throws_if_an_invalid_charset_is_specified() {
        let result = parse_with(
            "a=b",
            build_options(|opts| {
                opts.additional
                    .insert("charset".to_string(), QsValue::String("foobar".to_string()));
            }),
        );

        assert!(result.is_err());
    }

    // original: parses an iso-8859-1 string if asked to
    #[test]
    fn parses_an_iso_8859_1_string_if_asked_to() {
        assert_parse(
            "%A2=%BD",
            Some(build_options(|opts| {
                opts.charset = Charset::Iso88591;
            })),
            from_json(json!({ "¢": "½" })),
        );
    }

    // original: prefers an utf-8 charset specified by the utf8 sentinel to a default charset of iso-8859-1
    #[test]
    fn prefers_an_utf_8_charset_specified_by_the_utf8_sentinel_to_a_default_charset_of_iso_8859_1()
    {
        assert_parse(
            "utf8=%E2%9C%93&%C3%B8=%C3%B8",
            Some(build_options(|opts| {
                opts.charset_sentinel = true;
                opts.charset = Charset::Iso88591;
            })),
            from_json(json!({ "ø": "ø" })),
        );
    }

    // original: prefers an iso-8859-1 charset specified by the utf8 sentinel to a default charset of utf-8
    #[test]
    fn prefers_an_iso_8859_1_charset_specified_by_the_utf8_sentinel_to_a_default_charset_of_utf_8()
    {
        assert_parse(
            "utf8=%26%2310003%3B&%C3%B8=%C3%B8",
            Some(build_options(|opts| {
                opts.charset_sentinel = true;
                opts.charset = Charset::Utf8;
            })),
            from_json(json!({ "Ã¸": "Ã¸" })),
        );
    }

    // original: does not require the utf8 sentinel to be defined before the parameters whose decoding it affects
    #[test]
    fn does_not_require_the_utf8_sentinel_to_be_defined_before_the_parameters_whose_decoding_it_affects()
     {
        assert_parse(
            "a=%C3%B8&utf8=%26%2310003%3B",
            Some(build_options(|opts| {
                opts.charset_sentinel = true;
                opts.charset = Charset::Utf8;
            })),
            from_json(json!({ "a": "Ã¸" })),
        );
    }

    // original: ignores an utf8 sentinel with an unknown value
    #[test]
    fn ignores_an_utf8_sentinel_with_an_unknown_value() {
        assert_parse(
            "utf8=foo&%C3%B8=%C3%B8",
            Some(build_options(|opts| {
                opts.charset_sentinel = true;
                opts.charset = Charset::Utf8;
            })),
            from_json(json!({ "ø": "ø" })),
        );
    }

    // original: uses the utf8 sentinel to switch to utf-8 when no default charset is given
    #[test]
    fn uses_the_utf8_sentinel_to_switch_to_utf_8_when_no_default_charset_is_given() {
        assert_parse(
            "utf8=%E2%9C%93&%C3%B8=%C3%B8",
            Some(build_options(|opts| {
                opts.charset_sentinel = true;
            })),
            from_json(json!({ "ø": "ø" })),
        );
    }

    // original: uses the utf8 sentinel to switch to iso-8859-1 when no default charset is given
    #[test]
    fn uses_the_utf8_sentinel_to_switch_to_iso_8859_1_when_no_default_charset_is_given() {
        assert_parse(
            "utf8=%26%2310003%3B&%C3%B8=%C3%B8",
            Some(build_options(|opts| {
                opts.charset_sentinel = true;
            })),
            from_json(json!({ "Ã¸": "Ã¸" })),
        );
    }

    // original: interprets numeric entities in iso-8859-1 when `interpretNumericEntities`
    #[test]
    fn interprets_numeric_entities_in_iso_8859_1_when_interpretnumericentities() {
        assert_parse(
            "foo=%26%239786%3B",
            Some(build_options(|opts| {
                opts.charset = Charset::Iso88591;
                opts.interpret_numeric_entities = true;
            })),
            from_json(json!({ "foo": "☺" })),
        );
    }

    // original: handles a custom decoder returning `null`, in the `iso-8859-1` charset, when `interpretNumericEntities`
    #[test]
    fn handles_a_custom_decoder_returning_null_in_the_iso_8859_1_charset_when_interpretnumericentities()
     {
        assert_parse(
            "foo=&bar=%26%239786%3B",
            Some(build_options(|opts| {
                opts.charset = Charset::Iso88591;
                opts.interpret_numeric_entities = true;
                opts.decoder = Some(std::sync::Arc::new(
                    |input, default_decoder, charset, kind| {
                        if input.is_empty() {
                            QsValue::Null
                        } else {
                            default_decoder(input, charset, kind)
                        }
                    },
                ));
            })),
            from_json(json!({ "foo": null, "bar": "☺" })),
        );
    }

    // original: handles a custom decoder returning `null`, with a string key of `null`
    #[test]
    fn handles_a_custom_decoder_returning_null_with_a_string_key_of_null() {
        assert_parse(
            "null=1&ToNull=2",
            Some(build_options(|opts| {
                opts.decoder = Some(std::sync::Arc::new(
                    |input, default_decoder, charset, kind| {
                        if input == "ToNull" {
                            QsValue::Null
                        } else {
                            default_decoder(input, charset, kind)
                        }
                    },
                ));
            })),
            from_json(json!({ "null": "1" })),
        );
    }

    // original: does not interpret numeric entities in iso-8859-1 when `interpretNumericEntities` is absent
    #[test]
    fn does_not_interpret_numeric_entities_in_iso_8859_1_when_interpretnumericentities_is_absent() {
        assert_parse(
            "foo=%26%239786%3B",
            Some(build_options(|opts| {
                opts.charset = Charset::Iso88591;
            })),
            from_json(json!({ "foo": "&#9786;" })),
        );
    }

    // original: does not interpret numeric entities when the charset is utf-8, even when `interpretNumericEntities`
    #[test]
    fn does_not_interpret_numeric_entities_when_the_charset_is_utf_8_even_when_interpretnumericentities()
     {
        assert_parse(
            "foo=%26%239786%3B",
            Some(build_options(|opts| {
                opts.charset = Charset::Utf8;
                opts.interpret_numeric_entities = true;
            })),
            from_json(json!({ "foo": "&#9786;" })),
        );
    }

    // original: interpretNumericEntities with comma:true and iso charset does not crash
    #[test]
    fn interpretnumericentities_with_comma_true_and_iso_charset_does_not_crash() {
        assert_parse(
            "b&a[]=1,%26%239786%3B",
            Some(build_options(|opts| {
                opts.comma = true;
                opts.charset = Charset::Iso88591;
                opts.interpret_numeric_entities = true;
            })),
            from_json(json!({ "b": "", "a": ["1,☺"] })),
        );
    }

    // original: does not interpret %uXXXX syntax in iso-8859-1 mode
    #[test]
    fn does_not_interpret_uxxxx_syntax_in_iso_8859_1_mode() {
        assert_parse(
            "%u263A=%u263A",
            Some(build_options(|opts| {
                opts.charset = Charset::Iso88591;
            })),
            from_json(json!({ "%u263A": "%u263A" })),
        );
    }

    // original: allows for decoding keys and values differently
    #[test]
    fn allows_for_decoding_keys_and_values_differently() {
        assert_parse(
            "KeY=vAlUe",
            Some(build_options(|opts| {
                opts.decoder = Some(std::sync::Arc::new(
                    |input, default_decoder, charset, kind| {
                        let decoded = default_decoder(input, charset, kind);
                        match (kind, decoded) {
                            (ValueKind::Key, QsValue::String(s)) => {
                                QsValue::String(s.to_lowercase())
                            }
                            (ValueKind::Value, QsValue::String(s)) => {
                                QsValue::String(s.to_uppercase())
                            }
                            (_, other) => other,
                        }
                    },
                ));
            })),
            from_json(json!({ "key": "VALUE" })),
        );
    }

    // original: parameter limit tests
    #[test]
    fn parameter_limit_tests() {
        assert_parse(
            "a=1&b=2&c=3",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(5);
                opts.throw_on_limit_exceeded = Some(true);
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3"
            })),
        );

        let invalid_throw = parse_with(
            "a=1&b=2&c=3&d=4&e=5&f=6",
            build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
                opts.additional.insert(
                    "throwOnLimitExceeded".to_string(),
                    QsValue::String("true".to_string()),
                );
            }),
        );
        assert!(invalid_throw.is_err());

        let limit_exceeded = parse_with(
            "a=1&b=2&c=3&d=4&e=5&f=6",
            build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
                opts.throw_on_limit_exceeded = Some(true);
            }),
        );
        assert!(limit_exceeded.is_err());

        assert_parse(
            "a=1&b=2&c=3&d=4&e=5",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3"
            })),
        );

        assert_parse(
            "a=1&b=2&c=3&d=4&e=5",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
                opts.throw_on_limit_exceeded = Some(false);
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3"
            })),
        );

        assert_parse(
            "a=1&b=2&c=3&d=4&e=5&f=6",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Infinite;
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3",
                "d": "4",
                "e": "5",
                "f": "6"
            })),
        );
    }

    // original: does not throw error when within parameter limit
    #[test]
    fn does_not_throw_error_when_within_parameter_limit() {
        assert_parse(
            "a=1&b=2&c=3",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(5);
                opts.throw_on_limit_exceeded = Some(true);
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3"
            })),
        );
    }

    // original: throws error when throwOnLimitExceeded is present but not boolean
    #[test]
    fn throws_error_when_throwonlimitexceeded_is_present_but_not_boolean() {
        let result = parse_with(
            "a=1&b=2&c=3&d=4&e=5&f=6",
            build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
                opts.additional.insert(
                    "throwOnLimitExceeded".to_string(),
                    QsValue::String("true".to_string()),
                );
            }),
        );

        assert!(result.is_err());
    }

    // original: throws error when parameter limit exceeded
    #[test]
    fn throws_error_when_parameter_limit_exceeded() {
        let result = parse_with(
            "a=1&b=2&c=3&d=4&e=5&f=6",
            build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
                opts.throw_on_limit_exceeded = Some(true);
            }),
        );

        assert!(result.is_err());
    }

    // original: silently truncates when throwOnLimitExceeded is not given
    #[test]
    fn silently_truncates_when_throwonlimitexceeded_is_not_given() {
        assert_parse(
            "a=1&b=2&c=3&d=4&e=5",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3"
            })),
        );
    }

    // original: silently truncates when parameter limit exceeded without error
    #[test]
    fn silently_truncates_when_parameter_limit_exceeded_without_error() {
        assert_parse(
            "a=1&b=2&c=3&d=4&e=5",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Finite(3);
                opts.throw_on_limit_exceeded = Some(false);
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3"
            })),
        );
    }

    // original: allows unlimited parameters when parameterLimit set to Infinity
    #[test]
    fn allows_unlimited_parameters_when_parameterlimit_set_to_infinity() {
        assert_parse(
            "a=1&b=2&c=3&d=4&e=5&f=6",
            Some(build_options(|opts| {
                opts.parameter_limit = LimitSetting::Infinite;
            })),
            from_json(json!({
                "a": "1",
                "b": "2",
                "c": "3",
                "d": "4",
                "e": "5",
                "f": "6"
            })),
        );
    }

    // original: array limit tests
    #[test]
    fn array_limit_tests() {
        assert_parse(
            "a[]=1&a[]=2&a[]=3",
            Some(build_options(|opts| {
                opts.array_limit = 5;
                opts.throw_on_limit_exceeded = Some(true);
            })),
            from_json(json!({ "a": ["1", "2", "3"] })),
        );

        let invalid_throw = parse_with(
            "a[]=1&a[]=2&a[]=3&a[]=4",
            build_options(|opts| {
                opts.array_limit = 3;
                opts.additional.insert(
                    "throwOnLimitExceeded".to_string(),
                    QsValue::String("true".to_string()),
                );
            }),
        );
        assert!(invalid_throw.is_err());

        let limit_exceeded = parse_with(
            "a[]=1&a[]=2&a[]=3&a[]=4",
            build_options(|opts| {
                opts.array_limit = 3;
                opts.throw_on_limit_exceeded = Some(true);
            }),
        );
        assert!(limit_exceeded.is_err());

        assert_parse(
            "a[1]=1&a[2]=2&a[3]=3&a[4]=4&a[5]=5&a[6]=6",
            Some(build_options(|opts| {
                opts.array_limit = 5;
            })),
            from_json(json!({
                "a": {
                    "1": "1",
                    "2": "2",
                    "3": "3",
                    "4": "4",
                    "5": "5",
                    "6": "6"
                }
            })),
        );
    }

    // original: does not throw error when array is within limit
    #[test]
    fn does_not_throw_error_when_array_is_within_limit() {
        assert_parse(
            "a[]=1&a[]=2&a[]=3",
            Some(build_options(|opts| {
                opts.array_limit = 5;
                opts.throw_on_limit_exceeded = Some(true);
            })),
            from_json(json!({ "a": ["1", "2", "3"] })),
        );
    }

    // original: throws error when throwOnLimitExceeded is present but not boolean for array limit
    #[test]
    fn throws_error_when_throwonlimitexceeded_is_present_but_not_boolean_for_array_limit() {
        let result = parse_with(
            "a[]=1&a[]=2&a[]=3&a[]=4",
            build_options(|opts| {
                opts.array_limit = 3;
                opts.additional.insert(
                    "throwOnLimitExceeded".to_string(),
                    QsValue::String("true".to_string()),
                );
            }),
        );

        assert!(result.is_err());
    }

    // original: throws error when array limit exceeded
    #[test]
    fn throws_error_when_array_limit_exceeded() {
        let result = parse_with(
            "a[]=1&a[]=2&a[]=3&a[]=4",
            build_options(|opts| {
                opts.array_limit = 3;
                opts.throw_on_limit_exceeded = Some(true);
            }),
        );

        assert!(result.is_err());
    }

    // original: converts array to object if length is greater than limit
    #[test]
    fn converts_array_to_object_if_length_is_greater_than_limit() {
        assert_parse(
            "a[1]=1&a[2]=2&a[3]=3&a[4]=4&a[5]=5&a[6]=6",
            Some(build_options(|opts| {
                opts.array_limit = 5;
            })),
            from_json(json!({
                "a": {
                    "1": "1",
                    "2": "2",
                    "3": "3",
                    "4": "4",
                    "5": "5",
                    "6": "6"
                }
            })),
        );
    }
}

// original: parses empty keys
mod parses_empty_keys {
    use super::{assert_parse_default, from_json};
    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Deserialize)]
    struct EmptyKeysCase {
        input: String,
        #[serde(rename = "noEmptyKeys")]
        no_empty_keys: Value,
    }

    #[derive(Deserialize)]
    struct EmptyKeysFixture {
        #[serde(rename = "emptyTestCases")]
        cases: Vec<EmptyKeysCase>,
    }

    // original: skips empty string key with
    #[test]
    fn skips_empty_string_key_with() {
        let data = include_str!("empty-keys-cases.json");
        let fixtures: EmptyKeysFixture =
            serde_json::from_str(data).expect("failed to parse empty-keys-cases.json");

        for case in fixtures.cases {
            assert_parse_default(&case.input, from_json(case.no_empty_keys));
        }
    }
}

// original: `duplicates` option
mod duplicates_option {
    use super::{
        assert_parse, build_options, from_json, json, make_array, make_object, parse_with,
    };
    use bunner_qs::QsValue;

    // original: `duplicates` option
    #[test]
    fn duplicates_option_behaviour() {
        let invalid_values = vec![
            QsValue::Null,
            QsValue::Bool(true),
            QsValue::Bool(false),
            QsValue::Number(0.0),
            QsValue::Number(f64::NAN),
            make_array(vec![]),
            make_object(vec![]),
            QsValue::String("not a valid option".to_string()),
        ];

        for invalid in invalid_values {
            let result = parse_with(
                "",
                build_options(|opts| {
                    opts.additional
                        .insert("duplicates".to_string(), invalid.clone());
                }),
            );

            assert!(
                result.is_err(),
                "expected error for invalid duplicates option {invalid:?}"
            );
        }

        assert_parse(
            "foo=bar&foo=baz",
            None,
            from_json(json!({ "foo": ["bar", "baz"] })),
        );

        assert_parse(
            "foo=bar&foo=baz",
            Some(build_options(|opts| {
                opts.additional.insert(
                    "duplicates".to_string(),
                    QsValue::String("combine".to_string()),
                );
            })),
            from_json(json!({ "foo": ["bar", "baz"] })),
        );

        assert_parse(
            "foo=bar&foo=baz",
            Some(build_options(|opts| {
                opts.additional.insert(
                    "duplicates".to_string(),
                    QsValue::String("first".to_string()),
                );
            })),
            from_json(json!({ "foo": "bar" })),
        );

        assert_parse(
            "foo=bar&foo=baz",
            Some(build_options(|opts| {
                opts.additional.insert(
                    "duplicates".to_string(),
                    QsValue::String("last".to_string()),
                );
            })),
            from_json(json!({ "foo": "baz" })),
        );
    }
}

// original: qs strictDepth option - throw cases
mod qs_strictdepth_option_throw_cases {
    use super::{build_options, parse_with};
    use bunner_qs::DepthSetting;

    // original: throws an exception when depth exceeds the limit with strictDepth: true
    #[test]
    fn throws_an_exception_when_depth_exceeds_the_limit_with_strictdepth_true() {
        let result = parse_with(
            "a[b][c][d][e][f][g][h][i]=j",
            build_options(|opts| {
                opts.depth = DepthSetting::Finite(1);
                opts.strict_depth = true;
            }),
        );

        assert!(result.is_err());
    }

    // original: throws an exception for multiple nested arrays with strictDepth: true
    #[test]
    fn throws_an_exception_for_multiple_nested_arrays_with_strictdepth_true() {
        let result = parse_with(
            "a[0][1][2][3][4]=b",
            build_options(|opts| {
                opts.depth = DepthSetting::Finite(3);
                opts.strict_depth = true;
            }),
        );

        assert!(result.is_err());
    }

    // original: throws an exception for nested objects and arrays with strictDepth: true
    #[test]
    fn throws_an_exception_for_nested_objects_and_arrays_with_strictdepth_true() {
        let result = parse_with(
            "a[b][c][0][d][e]=f",
            build_options(|opts| {
                opts.depth = DepthSetting::Finite(3);
                opts.strict_depth = true;
            }),
        );

        assert!(result.is_err());
    }

    // original: throws an exception for different types of values with strictDepth: true
    #[test]
    fn throws_an_exception_for_different_types_of_values_with_strictdepth_true() {
        let result = parse_with(
            "a[b][c][d][e]=true&a[b][c][d][f]=42",
            build_options(|opts| {
                opts.depth = DepthSetting::Finite(3);
                opts.strict_depth = true;
            }),
        );

        assert!(result.is_err());
    }
}

// original: qs strictDepth option - non-throw cases
mod qs_strictdepth_option_non_throw_cases {
    use super::{assert_parse, build_options, from_json, json, parse_with};
    use bunner_qs::DepthSetting;

    // original: when depth is 0 and strictDepth true, do not throw
    #[test]
    fn when_depth_is_0_and_strictdepth_true_do_not_throw() {
        let result = parse_with(
            "a[b][c][d][e]=true&a[b][c][d][f]=42",
            build_options(|opts| {
                opts.depth = DepthSetting::Finite(0);
                opts.strict_depth = true;
            }),
        );

        assert!(result.is_ok());
    }

    // original: parses successfully when depth is within the limit with strictDepth: true
    #[test]
    fn parses_successfully_when_depth_is_within_the_limit_with_strictdepth_true() {
        assert_parse(
            "a[b]=c",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(1);
                opts.strict_depth = true;
            })),
            from_json(json!({
                "a": { "b": "c" }
            })),
        );
    }

    // original: does not throw an exception when depth exceeds the limit with strictDepth: false
    #[test]
    fn does_not_throw_an_exception_when_depth_exceeds_the_limit_with_strictdepth_false() {
        assert_parse(
            "a[b][c][d][e][f][g][h][i]=j",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(1);
            })),
            from_json(json!({
                "a": {
                    "b": {
                        "[c][d][e][f][g][h][i]": "j"
                    }
                }
            })),
        );
    }

    // original: parses successfully when depth is within the limit with strictDepth: false
    #[test]
    fn parses_successfully_when_depth_is_within_the_limit_with_strictdepth_false() {
        assert_parse(
            "a[b]=c",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(1);
            })),
            from_json(json!({
                "a": { "b": "c" }
            })),
        );
    }

    // original: does not throw when depth is exactly at the limit with strictDepth: true
    #[test]
    fn does_not_throw_when_depth_is_exactly_at_the_limit_with_strictdepth_true() {
        assert_parse(
            "a[b][c]=d",
            Some(build_options(|opts| {
                opts.depth = DepthSetting::Finite(2);
                opts.strict_depth = true;
            })),
            from_json(json!({
                "a": { "b": { "c": "d" } }
            })),
        );
    }
}
