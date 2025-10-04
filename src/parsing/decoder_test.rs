use super::*;
use assert_matches::assert_matches;
use std::borrow::Cow;

fn scratch_vec() -> Vec<u8> {
    Vec::new()
}

mod decode_component {
    use super::*;

    #[test]
    fn should_return_borrowed_slice_when_input_is_plain_ascii_then_avoid_allocation() {
        let raw = "simple";
        let mut scratch = super::scratch_vec();

        let result = decode_component(raw, false, 0, &mut scratch).expect("decode ascii");

        assert_matches!(result, Cow::Borrowed("simple"));
    }

    #[test]
    fn should_decode_plus_signs_as_spaces_when_space_as_plus_is_enabled_then_convert_plus_to_space()
    {
        let raw = "one+two";
        let mut scratch = super::scratch_vec();

        let result = decode_component(raw, true, 5, &mut scratch).expect("decode plus");

        assert_matches!(result, Cow::Owned(string) if string == "one two");
    }

    #[test]
    fn should_return_invalid_percent_error_when_second_hex_digit_is_invalid_then_report_index() {
        let raw = "%2G";
        let mut scratch = super::scratch_vec();

        let error = decode_component(raw, false, 12, &mut scratch)
            .expect_err("invalid second hex digit should fail");

        assert_matches!(
            error,
            ParseError::InvalidPercentEncoding { index } => {
                assert_eq!(index, 12);
            }
        );
    }

    #[test]
    fn should_return_invalid_character_error_when_control_character_is_present_then_report_character_and_index()
     {
        let raw = "bad\u{0007}";
        let mut scratch = super::scratch_vec();

        let error = decode_component(raw, false, 3, &mut scratch).expect_err("control char");

        assert_matches!(
            error,
            ParseError::InvalidCharacter { character, index } => {
                assert_eq!(character, '\u{0007}');
                assert_eq!(index, 6);
            }
        );
    }
}

mod fast_path_ascii {
    use super::*;

    #[test]
    fn should_return_borrowed_result_when_all_bytes_visible_then_avoid_allocation() {
        let raw = "visible";
        let outcome =
            fast_path_ascii_for_test(raw, raw.as_bytes(), 0).expect("fast path should borrow");
        assert_matches!(outcome, Cow::Borrowed("visible"));
    }

    #[test]
    fn should_error_when_control_character_detected_then_return_decode_error() {
        let raw = "bad\u{0007}";
        let err = fast_path_ascii_for_test(raw, raw.as_bytes(), 10)
            .expect_err("control characters should error");

        assert_matches!(
            err,
            ParseError::InvalidCharacter { character, index } => {
                assert_eq!(character, '\u{0007}');
                assert_eq!(index, 13);
            }
        );
    }
}

mod decode_with_special_chars {
    use super::*;

    #[test]
    fn should_decode_percent_sequences_and_plus_runs_when_mixed_input_present_then_normalize_bytes()
    {
        let raw = "%2B+matrix";
        let mut scratch = super::scratch_vec();

        let result = decode_with_special_chars_for_test(raw, raw.as_bytes(), true, 0, &mut scratch)
            .expect("percent and plus decode");

        assert_matches!(result, Cow::Owned(text) if text == "+ matrix");
    }

    #[test]
    fn should_propagate_invalid_percent_error_from_sequence_helper_when_sequence_invalid_then_bubble_error()
     {
        let raw = "%2Z";
        let mut scratch = super::scratch_vec();

        let err = decode_with_special_chars_for_test(raw, raw.as_bytes(), false, 4, &mut scratch)
            .expect_err("invalid percent should bubble up");

        assert_matches!(err, ParseError::InvalidPercentEncoding { index: 4 });
    }

    #[test]
    fn should_handle_utf8_cluster_when_non_ascii_byte_encountered_then_decode_cluster() {
        let raw = "café";
        let mut scratch = super::scratch_vec();

        let result =
            decode_with_special_chars_for_test(raw, raw.as_bytes(), false, 0, &mut scratch)
                .expect("utf8 cluster should decode");

        assert_matches!(result, Cow::Borrowed("café"));
    }
}

mod decode_percent_sequence {
    use super::*;

    #[test]
    fn should_push_decoded_byte_when_valid_percent_sequence_processed_then_return_next_cursor() {
        let bytes = b"%2A";
        let mut scratch = super::scratch_vec();

        let next =
            decode_percent_sequence_for_test(bytes, 0, 0, &mut scratch).expect("percent sequence");

        assert_eq!(next, 3);
        assert_eq!(scratch, vec![b'*']);
    }

    #[test]
    fn should_error_when_sequence_truncated_then_return_truncated_sequence_error() {
        let bytes = b"%2";
        let mut scratch = super::scratch_vec();

        let err = decode_percent_sequence_for_test(bytes, 0, 2, &mut scratch)
            .expect_err("truncated percent should err");

        assert_matches!(err, ParseError::InvalidPercentEncoding { index: 2 });
    }

    #[test]
    fn should_error_when_hex_digit_invalid_then_return_invalid_hex_error() {
        let bytes = b"%4Z";
        let mut scratch = super::scratch_vec();

        let err = decode_percent_sequence_for_test(bytes, 0, 7, &mut scratch)
            .expect_err("invalid hex digit should err");

        assert_matches!(err, ParseError::InvalidPercentEncoding { index: 7 });
    }
}

mod decode_plus {
    use super::*;

    #[test]
    fn should_write_space_to_scratch_when_plus_flag_enabled_then_increment_cursor() {
        let mut scratch = super::scratch_vec();

        let next = decode_plus_for_test(3, &mut scratch);

        assert_eq!(next, 4);
        assert_eq!(scratch, vec![b' ']);
    }
}

mod decode_ascii_run {
    use super::*;

    #[test]
    fn should_collect_ascii_until_percent_boundary_when_visible_run_present_then_accumulate_segment()
     {
        let bytes = b"abc%20";
        let mut scratch = super::scratch_vec();

        let next = decode_ascii_run_for_test(bytes, 0, 0, false, &mut scratch)
            .expect("ascii run should succeed");

        assert_eq!(next, 3);
        assert_eq!(scratch, b"abc");
    }

    #[test]
    fn should_stop_when_plus_encountered_and_space_flag_enabled_then_return_current_cursor() {
        let bytes = b"pre+more";
        let mut scratch = super::scratch_vec();

        let next = decode_ascii_run_for_test(bytes, 0, 0, true, &mut scratch)
            .expect("ascii run should stop at plus");

        assert_eq!(next, 3);
        assert_eq!(scratch, b"pre");
    }

    #[test]
    fn should_error_when_control_character_present_in_run_then_return_decode_error() {
        let bytes = b"ok\x07";
        let mut scratch = super::scratch_vec();

        let err = decode_ascii_run_for_test(bytes, 0, 5, false, &mut scratch)
            .expect_err("control char should fail");

        assert_matches!(
            err,
            ParseError::InvalidCharacter { character, index } => {
                assert_eq!(character, '\u{0007}');
                assert_eq!(index, 7);
            }
        );
    }
}

mod decode_utf8_cluster {
    use super::*;

    #[test]
    fn should_copy_utf8_sequence_when_multibyte_chunk_detected_then_return_next_cursor() {
        let raw = "😊 rest";
        let mut scratch = super::scratch_vec();

        let next = decode_utf8_cluster_for_test(raw, raw.as_bytes(), 0, &mut scratch)
            .expect("utf8 cluster should succeed");

        assert_eq!(next, "😊".len());
        assert_eq!(scratch, "😊".as_bytes());
    }

    #[test]
    fn should_error_when_cursor_out_of_bounds_then_return_decode_error() {
        let raw = "data";
        let mut scratch = super::scratch_vec();

        let err = decode_utf8_cluster_for_test(raw, raw.as_bytes(), raw.len(), &mut scratch)
            .expect_err("out of bounds should error");

        assert_matches!(err, ParseError::InvalidUtf8);
    }
}

mod finalize_decoded {
    use super::*;

    #[test]
    fn should_return_owned_string_when_bytes_are_valid_utf8_then_collect_string() {
        let mut scratch = b"hello".to_vec();

        let result = finalize_decoded_for_test(&mut scratch).expect("valid utf8");

        assert_matches!(result, Cow::Owned(text) if text == "hello");
        assert!(scratch.capacity() >= 5);
    }

    #[test]
    fn should_return_invalid_utf8_error_when_bytes_are_invalid_utf8_then_restore_cursor_state() {
        let mut scratch = vec![0xF0, 0x28, 0x8C, 0x28];

        let err = finalize_decoded_for_test(&mut scratch).expect_err("invalid utf8");

        assert_matches!(err, ParseError::InvalidUtf8);
        assert_eq!(scratch, vec![0xF0, 0x28, 0x8C, 0x28]);
    }
}

mod ensure_visible {
    use super::*;

    #[test]
    fn should_allow_visible_ascii_character_when_byte_is_visible_then_return_true() {
        ensure_visible_for_test(b'A', 0).expect("visible char should succeed");
    }

    #[test]
    fn should_error_for_control_character_when_control_byte_is_provided_then_return_false() {
        let err = ensure_visible_for_test(0x1F, 42).expect_err("control char should error");

        match err {
            ParseError::InvalidCharacter { character, index } => {
                assert_eq!(character, '\u{001F}');
                assert_eq!(index, 42);
            }
            other => panic!("expected InvalidCharacter, got {other:?}"),
        }
    }
}

mod hex_value {
    use super::*;

    #[test]
    fn should_decode_numeric_and_hex_letters_when_character_is_hex_digit_then_return_value() {
        assert_eq!(hex_value_for_test(b'0'), Some(0));
        assert_eq!(hex_value_for_test(b'9'), Some(9));
        assert_eq!(hex_value_for_test(b'a'), Some(10));
        assert_eq!(hex_value_for_test(b'F'), Some(15));
    }

    #[test]
    fn should_return_none_for_non_hex_character_when_character_is_not_hex_digit_then_return_none() {
        assert_eq!(hex_value_for_test(b'G'), None);
    }
}
