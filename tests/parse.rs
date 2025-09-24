//! Auto-generated skeleton from qs/test/parse.js
#![allow(unused)]

mod common;

use common::*;
use serde_json::json;

// original: parse()
mod parse {
    use super::{
        assert_parse, assert_parse_default, build_options, bytes, from_json, json, make_array,
        make_object, parse_default, parse_with,
    };
    use bunner_qs::{
        Charset, Delimiter, DepthSetting, DuplicateStrategy, LimitSetting, ParseOptions, QsValue,
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
    fn comma_false() {}

    // original: comma: true
    #[test]
    fn comma_true() {}

    // original: allows enabling dot notation
    #[test]
    fn allows_enabling_dot_notation() {}

    // original: decode dot keys correctly
    #[test]
    fn decode_dot_keys_correctly() {}

    // original: decodes dot in key of object, and allow enabling dot notation when decodeDotInKeys is set to true and allowDots is undefined
    #[test]
    fn decodes_dot_in_key_of_object_and_allow_enabling_dot_notation_when_decodedotinkeys_is_set_to_true_and_allowdots_is_undefined()
     {
    }

    // original: throws when decodeDotInKeys is not of type boolean
    #[test]
    fn throws_when_decodedotinkeys_is_not_of_type_boolean() {}

    // original: allows empty arrays in obj values
    #[test]
    fn allows_empty_arrays_in_obj_values() {}

    // original: throws when allowEmptyArrays is not of type boolean
    #[test]
    fn throws_when_allowemptyarrays_is_not_of_type_boolean() {}

    // original: allowEmptyArrays + strictNullHandling
    #[test]
    fn allowemptyarrays_strictnullhandling() {}

    // original: only parses one level when depth = 1
    #[test]
    fn only_parses_one_level_when_depth_1() {}

    // original: uses original key when depth = 0
    #[test]
    fn uses_original_key_when_depth_0() {}

    // original: uses original key when depth = false
    #[test]
    fn uses_original_key_when_depth_false() {}

    // original: parses an explicit array
    #[test]
    fn parses_an_explicit_array() {}

    // original: parses a mix of simple and explicit arrays
    #[test]
    fn parses_a_mix_of_simple_and_explicit_arrays() {}

    // original: parses a nested array
    #[test]
    fn parses_a_nested_array() {}

    // original: allows to specify array indices
    #[test]
    fn allows_to_specify_array_indices() {}

    // original: limits specific array indices to arrayLimit
    #[test]
    fn limits_specific_array_indices_to_arraylimit() {}

    // original: supports encoded = signs
    #[test]
    fn supports_encoded_signs() {}

    // original: is ok with url encoded strings
    #[test]
    fn is_ok_with_url_encoded_strings() {}

    // original: allows brackets in the value
    #[test]
    fn allows_brackets_in_the_value() {}

    // original: allows empty values
    #[test]
    fn allows_empty_values() {}

    // original: transforms arrays to objects
    #[test]
    fn transforms_arrays_to_objects() {}

    // original: transforms arrays to objects (dot notation)
    #[test]
    fn transforms_arrays_to_objects_dot_notation() {}

    // original: correctly prunes undefined values when converting an array to an object
    #[test]
    fn correctly_prunes_undefined_values_when_converting_an_array_to_an_object() {}

    // original: supports malformed uri characters
    #[test]
    fn supports_malformed_uri_characters() {}

    // original: doesn't produce empty keys
    #[test]
    fn doesn_t_produce_empty_keys() {}

    // original: cannot access Object prototype
    #[test]
    fn cannot_access_object_prototype() {}

    // original: parses arrays of objects
    #[test]
    fn parses_arrays_of_objects() {}

    // original: allows for empty strings in arrays
    #[test]
    fn allows_for_empty_strings_in_arrays() {}

    // original: compacts sparse arrays
    #[test]
    fn compacts_sparse_arrays() {}

    // original: parses sparse arrays
    #[test]
    fn parses_sparse_arrays() {}

    // original: parses semi-parsed strings
    #[test]
    fn parses_semi_parsed_strings() {}

    // original: parses buffers correctly
    #[test]
    fn parses_buffers_correctly() {}

    // original: parses jquery-param strings
    #[test]
    fn parses_jquery_param_strings() {}

    // original: continues parsing when no parent is found
    #[test]
    fn continues_parsing_when_no_parent_is_found() {}

    // original: does not error when parsing a very long array
    #[test]
    fn does_not_error_when_parsing_a_very_long_array() {}

    // original: does not throw when a native prototype has an enumerable property
    #[test]
    fn does_not_throw_when_a_native_prototype_has_an_enumerable_property() {}

    // original: parses a string with an alternative string delimiter
    #[test]
    fn parses_a_string_with_an_alternative_string_delimiter() {}

    // original: parses a string with an alternative RegExp delimiter
    #[test]
    fn parses_a_string_with_an_alternative_regexp_delimiter() {}

    // original: does not use non-splittable objects as delimiters
    #[test]
    fn does_not_use_non_splittable_objects_as_delimiters() {}

    // original: allows overriding parameter limit
    #[test]
    fn allows_overriding_parameter_limit() {}

    // original: allows setting the parameter limit to Infinity
    #[test]
    fn allows_setting_the_parameter_limit_to_infinity() {}

    // original: allows overriding array limit
    #[test]
    fn allows_overriding_array_limit() {}

    // original: allows disabling array parsing
    #[test]
    fn allows_disabling_array_parsing() {}

    // original: allows for query string prefix
    #[test]
    fn allows_for_query_string_prefix() {}

    // original: parses an object
    #[test]
    fn parses_an_object() {}

    // original: parses string with comma as array divider
    #[test]
    fn parses_string_with_comma_as_array_divider() {}

    // original: parses values with comma as array divider
    #[test]
    fn parses_values_with_comma_as_array_divider() {}

    // original: use number decoder, parses string that has one number with comma option enabled
    #[test]
    fn use_number_decoder_parses_string_that_has_one_number_with_comma_option_enabled() {}

    // original: parses brackets holds array of arrays when having two parts of strings with comma as array divider
    #[test]
    fn parses_brackets_holds_array_of_arrays_when_having_two_parts_of_strings_with_comma_as_array_divider()
     {
    }

    // original: parses url-encoded brackets holds array of arrays when having two parts of strings with comma as array divider
    #[test]
    fn parses_url_encoded_brackets_holds_array_of_arrays_when_having_two_parts_of_strings_with_comma_as_array_divider()
     {
    }

    // original: parses comma delimited array while having percent-encoded comma treated as normal text
    #[test]
    fn parses_comma_delimited_array_while_having_percent_encoded_comma_treated_as_normal_text() {}

    // original: parses an object in dot notation
    #[test]
    fn parses_an_object_in_dot_notation() {}

    // original: parses an object and not child values
    #[test]
    fn parses_an_object_and_not_child_values() {}

    // original: does not blow up when Buffer global is missing
    #[test]
    fn does_not_blow_up_when_buffer_global_is_missing() {}

    // original: does not crash when parsing circular references
    #[test]
    fn does_not_crash_when_parsing_circular_references() {}

    // original: does not crash when parsing deep objects
    #[test]
    fn does_not_crash_when_parsing_deep_objects() {}

    // original: parses null objects correctly
    #[test]
    fn parses_null_objects_correctly() {}

    // original: parses dates correctly
    #[test]
    fn parses_dates_correctly() {}

    // original: parses regular expressions correctly
    #[test]
    fn parses_regular_expressions_correctly() {}

    // original: does not allow overwriting prototype properties
    #[test]
    fn does_not_allow_overwriting_prototype_properties() {}

    // original: can allow overwriting prototype properties
    #[test]
    fn can_allow_overwriting_prototype_properties() {}

    // original: does not crash when the global Object prototype is frozen
    #[test]
    fn does_not_crash_when_the_global_object_prototype_is_frozen() {}

    // original: params starting with a closing bracket
    #[test]
    fn params_starting_with_a_closing_bracket() {}

    // original: params starting with a starting bracket
    #[test]
    fn params_starting_with_a_starting_bracket() {}

    // original: add keys to objects
    #[test]
    fn add_keys_to_objects() {}

    // original: dunder proto is ignored
    #[test]
    fn dunder_proto_is_ignored() {}

    // original: can return null objects
    #[test]
    fn can_return_null_objects() {}

    // original: can parse with custom encoding
    #[test]
    fn can_parse_with_custom_encoding() {}

    // original: receives the default decoder as a second argument
    #[test]
    fn receives_the_default_decoder_as_a_second_argument() {}

    // original: throws error with wrong decoder
    #[test]
    fn throws_error_with_wrong_decoder() {}

    // original: does not mutate the options argument
    #[test]
    fn does_not_mutate_the_options_argument() {}

    // original: throws if an invalid charset is specified
    #[test]
    fn throws_if_an_invalid_charset_is_specified() {}

    // original: parses an iso-8859-1 string if asked to
    #[test]
    fn parses_an_iso_8859_1_string_if_asked_to() {}

    // original: prefers an utf-8 charset specified by the utf8 sentinel to a default charset of iso-8859-1
    #[test]
    fn prefers_an_utf_8_charset_specified_by_the_utf8_sentinel_to_a_default_charset_of_iso_8859_1()
    {
    }

    // original: prefers an iso-8859-1 charset specified by the utf8 sentinel to a default charset of utf-8
    #[test]
    fn prefers_an_iso_8859_1_charset_specified_by_the_utf8_sentinel_to_a_default_charset_of_utf_8()
    {
    }

    // original: does not require the utf8 sentinel to be defined before the parameters whose decoding it affects
    #[test]
    fn does_not_require_the_utf8_sentinel_to_be_defined_before_the_parameters_whose_decoding_it_affects()
     {
    }

    // original: ignores an utf8 sentinel with an unknown value
    #[test]
    fn ignores_an_utf8_sentinel_with_an_unknown_value() {}

    // original: uses the utf8 sentinel to switch to utf-8 when no default charset is given
    #[test]
    fn uses_the_utf8_sentinel_to_switch_to_utf_8_when_no_default_charset_is_given() {}

    // original: uses the utf8 sentinel to switch to iso-8859-1 when no default charset is given
    #[test]
    fn uses_the_utf8_sentinel_to_switch_to_iso_8859_1_when_no_default_charset_is_given() {}

    // original: interprets numeric entities in iso-8859-1 when `interpretNumericEntities`
    #[test]
    fn interprets_numeric_entities_in_iso_8859_1_when_interpretnumericentities() {}

    // original: handles a custom decoder returning `null`, in the `iso-8859-1` charset, when `interpretNumericEntities`
    #[test]
    fn handles_a_custom_decoder_returning_null_in_the_iso_8859_1_charset_when_interpretnumericentities()
     {
    }

    // original: handles a custom decoder returning `null`, with a string key of `null`
    #[test]
    fn handles_a_custom_decoder_returning_null_with_a_string_key_of_null() {}

    // original: does not interpret numeric entities in iso-8859-1 when `interpretNumericEntities` is absent
    #[test]
    fn does_not_interpret_numeric_entities_in_iso_8859_1_when_interpretnumericentities_is_absent() {
    }

    // original: does not interpret numeric entities when the charset is utf-8, even when `interpretNumericEntities`
    #[test]
    fn does_not_interpret_numeric_entities_when_the_charset_is_utf_8_even_when_interpretnumericentities()
     {
    }

    // original: interpretNumericEntities with comma:true and iso charset does not crash
    #[test]
    fn interpretnumericentities_with_comma_true_and_iso_charset_does_not_crash() {}

    // original: does not interpret %uXXXX syntax in iso-8859-1 mode
    #[test]
    fn does_not_interpret_uxxxx_syntax_in_iso_8859_1_mode() {}

    // original: allows for decoding keys and values differently
    #[test]
    fn allows_for_decoding_keys_and_values_differently() {}

    // original: parameter limit tests
    #[test]
    fn parameter_limit_tests() {}

    // original: does not throw error when within parameter limit
    #[test]
    fn does_not_throw_error_when_within_parameter_limit() {}

    // original: throws error when throwOnLimitExceeded is present but not boolean
    #[test]
    fn throws_error_when_throwonlimitexceeded_is_present_but_not_boolean() {}

    // original: throws error when parameter limit exceeded
    #[test]
    fn throws_error_when_parameter_limit_exceeded() {}

    // original: silently truncates when throwOnLimitExceeded is not given
    #[test]
    fn silently_truncates_when_throwonlimitexceeded_is_not_given() {}

    // original: silently truncates when parameter limit exceeded without error
    #[test]
    fn silently_truncates_when_parameter_limit_exceeded_without_error() {}

    // original: allows unlimited parameters when parameterLimit set to Infinity
    #[test]
    fn allows_unlimited_parameters_when_parameterlimit_set_to_infinity() {}

    // original: array limit tests
    #[test]
    fn array_limit_tests() {}

    // original: does not throw error when array is within limit
    #[test]
    fn does_not_throw_error_when_array_is_within_limit() {}

    // original: throws error when throwOnLimitExceeded is present but not boolean for array limit
    #[test]
    fn throws_error_when_throwonlimitexceeded_is_present_but_not_boolean_for_array_limit() {}

    // original: throws error when array limit exceeded
    #[test]
    fn throws_error_when_array_limit_exceeded() {}

    // original: converts array to object if length is greater than limit
    #[test]
    fn converts_array_to_object_if_length_is_greater_than_limit() {}
}

// original: parses empty keys
mod parses_empty_keys {
    // original: skips empty string key with
    #[test]
    fn skips_empty_string_key_with() {}
}

// original: `duplicates` option
mod duplicates_option {}

// original: qs strictDepth option - throw cases
mod qs_strictdepth_option_throw_cases {
    // original: throws an exception when depth exceeds the limit with strictDepth: true
    #[test]
    fn throws_an_exception_when_depth_exceeds_the_limit_with_strictdepth_true() {}

    // original: throws an exception for multiple nested arrays with strictDepth: true
    #[test]
    fn throws_an_exception_for_multiple_nested_arrays_with_strictdepth_true() {}

    // original: throws an exception for nested objects and arrays with strictDepth: true
    #[test]
    fn throws_an_exception_for_nested_objects_and_arrays_with_strictdepth_true() {}

    // original: throws an exception for different types of values with strictDepth: true
    #[test]
    fn throws_an_exception_for_different_types_of_values_with_strictdepth_true() {}
}

// original: qs strictDepth option - non-throw cases
mod qs_strictdepth_option_non_throw_cases {
    // original: when depth is 0 and strictDepth true, do not throw
    #[test]
    fn when_depth_is_0_and_strictdepth_true_do_not_throw() {}

    // original: parses successfully when depth is within the limit with strictDepth: true
    #[test]
    fn parses_successfully_when_depth_is_within_the_limit_with_strictdepth_true() {}

    // original: does not throw an exception when depth exceeds the limit with strictDepth: false
    #[test]
    fn does_not_throw_an_exception_when_depth_exceeds_the_limit_with_strictdepth_false() {}

    // original: parses successfully when depth is within the limit with strictDepth: false
    #[test]
    fn parses_successfully_when_depth_is_within_the_limit_with_strictdepth_false() {}

    // original: does not throw when depth is exactly at the limit with strictDepth: true
    #[test]
    fn does_not_throw_when_depth_is_exactly_at_the_limit_with_strictdepth_true() {}
}
