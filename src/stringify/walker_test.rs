use super::Segment;

fn append(initial: &str, segment: Segment<'_>) -> String {
    let mut buffer = String::from(initial);
    super::append_segment(&mut buffer, segment);
    buffer
}

fn append_all(initial: &str, segments: &[Segment<'_>]) -> String {
    segments
        .iter()
        .fold(String::from(initial), |mut buffer, segment| {
            super::append_segment(&mut buffer, *segment);
            buffer
        })
}

mod append_segment {
    use super::*;

    #[test]
    fn should_write_key_for_root_segment_when_segment_is_root_then_write_segment_key() {
        let initial = "";

        let result = append(initial, Segment::Root("profile"));

        assert_eq!(result, "profile");
    }

    #[test]
    fn should_bracket_sub_key_for_object_segment_when_segment_is_object_then_append_bracketed_key()
    {
        let initial = "profile";

        let result = append(initial, Segment::Object("details"));

        assert_eq!(result, "profile[details]");
    }

    #[test]
    fn should_use_decimal_index_for_array_segment_when_segment_is_array_then_append_decimal_index()
    {
        let initial = "items";

        let result = append(initial, Segment::Array(42));

        assert_eq!(result, "items[42]");
    }

    #[test]
    fn should_append_zero_digit_for_zero_index_when_segment_index_is_zero_then_append_zero_digit() {
        let initial = "list";

        let result = append(initial, Segment::Array(0));

        assert_eq!(result, "list[0]");
    }

    #[test]
    fn should_build_full_key_path_when_chained_when_multiple_segments_are_appended_then_build_full_path()
     {
        let segments = [
            Segment::Root("order"),
            Segment::Object("items"),
            Segment::Array(7),
            Segment::Object("sku"),
        ];

        let result = append_all("", &segments);

        assert_eq!(result, "order[items][7][sku]");
    }
}

mod ascii_digits_to_str {
    #[test]
    fn should_convert_digit_bytes_to_str_when_bytes_are_numeric_then_return_string() {
        let digits = b"12345";

        let result = super::super::ascii_digits_to_str(digits);

        assert_eq!(result, "12345");
    }

    #[test]
    fn should_detect_invalid_digit_bytes_based_on_build_mode_when_invalid_digits_present_then_follow_build_guard()
     {
        let invalid = b"12a45";

        if cfg!(debug_assertions) {
            let panic_result = std::panic::catch_unwind(|| {
                let _ = super::super::ascii_digits_to_str(invalid);
            });
            assert!(panic_result.is_err(), "expected panic for invalid digits");
            return;
        }

        let result = super::super::ascii_digits_to_str(invalid);
        assert_eq!(result, "12a45");
    }
}
