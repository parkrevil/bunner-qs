//! Auto-generated skeleton from qs/test/stringify.js
#![allow(unused)]

// original: stringify()
mod stringify {
    // original: stringifies a querystring object
    #[test]
    fn stringifies_a_querystring_object() {}

    // original: stringifies falsy values
    #[test]
    fn stringifies_falsy_values() {}

    // original: stringifies symbols
    #[test]
    fn stringifies_symbols() {}

    // original: stringifies bigints
    #[test]
    fn stringifies_bigints() {}

    // original: encodes dot in key of object when encodeDotInKeys and allowDots is provided
    #[test]
    fn encodes_dot_in_key_of_object_when_encodedotinkeys_and_allowdots_is_provided() {}

    // original: should encode dot in key of object, and automatically set allowDots to `true` when encodeDotInKeys is true and allowDots in undefined
    #[test]
    fn should_encode_dot_in_key_of_object_and_automatically_set_allowdots_to_true_when_encodedotinkeys_is_true_and_allowdots_in_undefined()
     {
    }

    // original: should encode dot in key of object when encodeDotInKeys and allowDots is provided, and nothing else when encodeValuesOnly is provided
    #[test]
    fn should_encode_dot_in_key_of_object_when_encodedotinkeys_and_allowdots_is_provided_and_nothing_else_when_encodevaluesonly_is_provided()
     {
    }

    // original: throws when `commaRoundTrip` is not a boolean
    #[test]
    fn throws_when_commaroundtrip_is_not_a_boolean() {}

    // original: throws when `encodeDotInKeys` is not a boolean
    #[test]
    fn throws_when_encodedotinkeys_is_not_a_boolean() {}

    // original: adds query prefix
    #[test]
    fn adds_query_prefix() {}

    // original: with query prefix, outputs blank string given an empty object
    #[test]
    fn with_query_prefix_outputs_blank_string_given_an_empty_object() {}

    // original: stringifies nested falsy values
    #[test]
    fn stringifies_nested_falsy_values() {}

    // original: stringifies a nested object
    #[test]
    fn stringifies_a_nested_object() {}

    // original: `allowDots` option: stringifies a nested object with dots notation
    #[test]
    fn allowdots_option_stringifies_a_nested_object_with_dots_notation() {}

    // original: stringifies an array value
    #[test]
    fn stringifies_an_array_value() {}

    // original: `skipNulls` option
    #[test]
    fn skipnulls_option() {}

    // original: omits array indices when asked
    #[test]
    fn omits_array_indices_when_asked() {}

    // original: omits object key/value pair when value is empty array
    #[test]
    fn omits_object_key_value_pair_when_value_is_empty_array() {}

    // original: should not omit object key/value pair when value is empty array and when asked
    #[test]
    fn should_not_omit_object_key_value_pair_when_value_is_empty_array_and_when_asked() {}

    // original: should throw when allowEmptyArrays is not of type boolean
    #[test]
    fn should_throw_when_allowemptyarrays_is_not_of_type_boolean() {}

    // original: allowEmptyArrays + strictNullHandling
    #[test]
    fn allowemptyarrays_strictnullhandling() {}

    // original: stringifies an array value with one item vs multiple items
    #[test]
    fn stringifies_an_array_value_with_one_item_vs_multiple_items() {}

    // original: non-array item
    #[test]
    fn non_array_item() {}

    // original: array with a single item
    #[test]
    fn array_with_a_single_item() {}

    // original: array with multiple items
    #[test]
    fn array_with_multiple_items() {}

    // original: array with multiple items with a comma inside
    #[test]
    fn array_with_multiple_items_with_a_comma_inside() {}

    // original: stringifies a nested array value
    #[test]
    fn stringifies_a_nested_array_value() {}

    // original: stringifies comma and empty array values
    #[test]
    fn stringifies_comma_and_empty_array_values() {}

    // original: stringifies comma and empty non-array values
    #[test]
    fn stringifies_comma_and_empty_non_array_values() {}

    // original: stringifies a nested array value with dots notation
    #[test]
    fn stringifies_a_nested_array_value_with_dots_notation() {}

    // original: stringifies an object inside an array
    #[test]
    fn stringifies_an_object_inside_an_array() {}

    // original: stringifies an array with mixed objects and primitives
    #[test]
    fn stringifies_an_array_with_mixed_objects_and_primitives() {}

    // original: stringifies an object inside an array with dots notation
    #[test]
    fn stringifies_an_object_inside_an_array_with_dots_notation() {}

    // original: does not omit object keys when indices = false
    #[test]
    fn does_not_omit_object_keys_when_indices_false() {}

    // original: uses indices notation for arrays when indices=true
    #[test]
    fn uses_indices_notation_for_arrays_when_indices_true() {}

    // original: uses indices notation for arrays when no arrayFormat is specified
    #[test]
    fn uses_indices_notation_for_arrays_when_no_arrayformat_is_specified() {}

    // original: uses indices notation for arrays when arrayFormat=indices
    #[test]
    fn uses_indices_notation_for_arrays_when_arrayformat_indices() {}

    // original: uses repeat notation for arrays when arrayFormat=repeat
    #[test]
    fn uses_repeat_notation_for_arrays_when_arrayformat_repeat() {}

    // original: uses brackets notation for arrays when arrayFormat=brackets
    #[test]
    fn uses_brackets_notation_for_arrays_when_arrayformat_brackets() {}

    // original: stringifies a complicated object
    #[test]
    fn stringifies_a_complicated_object() {}

    // original: stringifies an empty value
    #[test]
    fn stringifies_an_empty_value() {}

    // original: stringifies an empty array in different arrayFormat
    #[test]
    fn stringifies_an_empty_array_in_different_arrayformat() {}

    // original: stringifies a null object
    #[test]
    fn stringifies_a_null_object() {}

    // original: returns an empty string for invalid input
    #[test]
    fn returns_an_empty_string_for_invalid_input() {}

    // original: stringifies an object with a null object as a child
    #[test]
    fn stringifies_an_object_with_a_null_object_as_a_child() {}

    // original: drops keys with a value of undefined
    #[test]
    fn drops_keys_with_a_value_of_undefined() {}

    // original: url encodes values
    #[test]
    fn url_encodes_values() {}

    // original: stringifies a date
    #[test]
    fn stringifies_a_date() {}

    // original: stringifies the weird object from qs
    #[test]
    fn stringifies_the_weird_object_from_qs() {}

    // original: skips properties that are part of the object prototype
    #[test]
    fn skips_properties_that_are_part_of_the_object_prototype() {}

    // original: stringifies boolean values
    #[test]
    fn stringifies_boolean_values() {}

    // original: stringifies buffer values
    #[test]
    fn stringifies_buffer_values() {}

    // original: stringifies an object using an alternative delimiter
    #[test]
    fn stringifies_an_object_using_an_alternative_delimiter() {}

    // original: does not blow up when Buffer global is missing
    #[test]
    fn does_not_blow_up_when_buffer_global_is_missing() {}

    // original: does not crash when parsing circular references
    #[test]
    fn does_not_crash_when_parsing_circular_references() {}

    // original: non-circular duplicated references can still work
    #[test]
    fn non_circular_duplicated_references_can_still_work() {}

    // original: selects properties when filter=array
    #[test]
    fn selects_properties_when_filter_array() {}

    // original: supports custom representations when filter=function
    #[test]
    fn supports_custom_representations_when_filter_function() {}

    // original: can disable uri encoding
    #[test]
    fn can_disable_uri_encoding() {}

    // original: can sort the keys
    #[test]
    fn can_sort_the_keys() {}

    // original: can sort the keys at depth 3 or more too
    #[test]
    fn can_sort_the_keys_at_depth_3_or_more_too() {}

    // original: can stringify with custom encoding
    #[test]
    fn can_stringify_with_custom_encoding() {}

    // original: receives the default encoder as a second argument
    #[test]
    fn receives_the_default_encoder_as_a_second_argument() {}

    // original: receives the default encoder as a second argument
    #[test]
    fn receives_the_default_encoder_as_a_second_argument_1() {}

    // original: throws error with wrong encoder
    #[test]
    fn throws_error_with_wrong_encoder() {}

    // original: can use custom encoder for a buffer object
    #[test]
    fn can_use_custom_encoder_for_a_buffer_object() {}

    // original: serializeDate option
    #[test]
    fn serializedate_option() {}

    // original: RFC 1738 serialization
    #[test]
    fn rfc_1738_serialization() {}

    // original: RFC 3986 spaces serialization
    #[test]
    fn rfc_3986_spaces_serialization() {}

    // original: Backward compatibility to RFC 3986
    #[test]
    fn backward_compatibility_to_rfc_3986() {}

    // original: Edge cases and unknown formats
    #[test]
    fn edge_cases_and_unknown_formats() {}

    // original: encodeValuesOnly
    #[test]
    fn encodevaluesonly() {}

    // original: encodeValuesOnly - strictNullHandling
    #[test]
    fn encodevaluesonly_strictnullhandling() {}

    // original: throws if an invalid charset is specified
    #[test]
    fn throws_if_an_invalid_charset_is_specified() {}

    // original: respects a charset of iso-8859-1
    #[test]
    fn respects_a_charset_of_iso_8859_1() {}

    // original: encodes unrepresentable chars as numeric entities in iso-8859-1 mode
    #[test]
    fn encodes_unrepresentable_chars_as_numeric_entities_in_iso_8859_1_mode() {}

    // original: respects an explicit charset of utf-8 (the default)
    #[test]
    fn respects_an_explicit_charset_of_utf_8_the_default() {}

    // original: `charsetSentinel` option
    #[test]
    fn charsetsentinel_option() {}

    // original: does not mutate the options argument
    #[test]
    fn does_not_mutate_the_options_argument() {}

    // original: strictNullHandling works with custom filter
    #[test]
    fn strictnullhandling_works_with_custom_filter() {}

    // original: strictNullHandling works with null serializeDate
    #[test]
    fn strictnullhandling_works_with_null_serializedate() {}

    // original: allows for encoding keys and values differently
    #[test]
    fn allows_for_encoding_keys_and_values_differently() {}

    // original: objects inside arrays
    #[test]
    fn objects_inside_arrays() {}

    // original: stringifies sparse arrays
    #[test]
    fn stringifies_sparse_arrays() {}

    // original: encodes a very long string
    #[test]
    fn encodes_a_very_long_string() {}
}

// original: stringifies empty keys
mod stringifies_empty_keys {
    // original: stringifies an object with empty string key with
    #[test]
    fn stringifies_an_object_with_empty_string_key_with() {}

    // original: edge case with object/arrays
    #[test]
    fn edge_case_with_object_arrays() {}

    // original: stringifies non-string keys
    #[test]
    fn stringifies_non_string_keys() {}
}
