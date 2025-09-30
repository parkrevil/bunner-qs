use super::{Segment, append_segment};

mod walker_tests {
    use super::{append_segment, Segment};

    #[test]
    fn when_appending_root_segment_it_should_write_key_directly() {
        let mut buffer = String::new();

        append_segment(&mut buffer, Segment::Root("profile"));

        assert_eq!(buffer, "profile");
    }

    #[test]
    fn when_appending_object_segment_it_should_bracket_sub_key() {
        let mut buffer = String::from("profile");

        append_segment(&mut buffer, Segment::Object("details"));

        assert_eq!(buffer, "profile[details]");
    }

    #[test]
    fn when_appending_array_segment_it_should_use_decimal_index() {
        let mut buffer = String::from("items");

        append_segment(&mut buffer, Segment::Array(42));

        assert_eq!(buffer, "items[42]");
    }

    #[test]
    fn when_array_index_is_zero_it_should_append_single_zero_digit() {
        let mut buffer = String::from("list");

        append_segment(&mut buffer, Segment::Array(0));

        assert_eq!(buffer, "list[0]");
    }

    #[test]
    fn when_chaining_segments_it_should_build_full_key_path() {
        let mut buffer = String::new();

        append_segment(&mut buffer, Segment::Root("order"));
        append_segment(&mut buffer, Segment::Object("items"));
        append_segment(&mut buffer, Segment::Array(7));
        append_segment(&mut buffer, Segment::Object("sku"));

        assert_eq!(buffer, "order[items][7][sku]");
    }
}
