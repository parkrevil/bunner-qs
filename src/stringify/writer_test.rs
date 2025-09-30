use super::write_pair;

mod write_pair_tests {
    use super::write_pair;

    #[test]
    fn when_writing_first_pair_it_should_not_prefix_ampersand() {
        // Arrange
        let mut output = String::new();
        let mut first_pair = true;

        // Act
        write_pair(&mut output, "user", "alice", false, &mut first_pair);

        // Assert
        assert_eq!(output, "user=alice");
        assert!(
            !first_pair,
            "first_pair flag should flip to false after first write"
        );
    }

    #[test]
    fn when_writing_subsequent_pair_it_should_prefix_separator() {
        // Arrange
        let mut output = String::from("first=one");
        let mut first_pair = false;

        // Act
        write_pair(
            &mut output,
            "second field",
            "two & two",
            false,
            &mut first_pair,
        );

        // Assert
        assert_eq!(
            output, "first=one&second%20field=two%20%26%20two",
            "second pair should be prefixed with '&' and percent encoded"
        );
        assert!(
            !first_pair,
            "first_pair flag should remain false for subsequent writes"
        );
    }

    #[test]
    fn when_space_as_plus_is_enabled_it_should_encode_spaces_as_plus() {
        // Arrange
        let mut output = String::new();
        let mut first_pair = true;

        // Act
        write_pair(
            &mut output,
            "space key",
            "space value",
            true,
            &mut first_pair,
        );

        // Assert
        assert_eq!(output, "space+key=space+value");
        assert!(!first_pair);
    }

    #[test]
    fn when_components_need_percent_encoding_it_should_escape_reserved_characters() {
        // Arrange
        let mut output = String::with_capacity(0);
        let mut first_pair = true;

        // Act
        write_pair(
            &mut output,
            "name+role?",
            "value/with=reserved&stuff",
            false,
            &mut first_pair,
        );

        // Assert
        assert_eq!(
            output, "name%2Brole%3F=value%2Fwith%3Dreserved%26stuff",
            "reserved characters should be percent encoded"
        );
        assert!(!first_pair);
        assert!(
            output.capacity() >= output.len(),
            "writer should ensure buffer has enough capacity"
        );
    }
}
