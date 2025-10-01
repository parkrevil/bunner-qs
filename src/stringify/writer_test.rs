// use super::write_pair; // use fully qualified path to avoid name clash with module name

struct WriteOutcome {
    output: String,
    first_pair: bool,
}

fn write_once(
    initial: &str,
    key: &str,
    value: &str,
    space_as_plus: bool,
    first_pair: bool,
) -> WriteOutcome {
    let mut output = String::from(initial);
    let mut first = first_pair;
    crate::stringify::writer::write_pair(&mut output, key, value, space_as_plus, &mut first);
    WriteOutcome {
        output,
        first_pair: first,
    }
}

mod write_pair {
    use super::*;

    #[test]
    fn should_avoid_ampersand_when_pair_is_first_then_write_pair_without_separator() {
        // Arrange
        let initial = "";
        let key = "user";
        let value = "alice";

        // Act
        let WriteOutcome { output, first_pair } = write_once(initial, key, value, false, true);

        // Assert
        assert_eq!(output, "user=alice");
        assert!(!first_pair);
    }

    #[test]
    fn should_prefix_separator_when_writing_subsequent_pair_then_append_ampersand() {
        // Arrange
        let initial = "first=one";
        let key = "second field";
        let value = "two & two";

        // Act
        let WriteOutcome { output, first_pair } = write_once(initial, key, value, false, false);

        // Assert
        assert_eq!(output, "first=one&second%20field=two%20%26%20two");
        assert!(!first_pair);
    }

    #[test]
    fn should_encode_spaces_as_plus_when_option_enabled_then_replace_spaces_with_plus() {
        // Arrange
        let initial = "";
        let key = "space key";
        let value = "space value";

        // Act
        let WriteOutcome { output, first_pair } = write_once(initial, key, value, true, true);

        // Assert
        assert_eq!(output, "space+key=space+value");
        assert!(!first_pair);
    }

    #[test]
    fn should_escape_reserved_characters_when_reserved_characters_present_then_percent_encode_reserved_chars() {
        // Arrange
        let mut output = String::with_capacity(0);
        let mut first_pair = true;

        // Act
        crate::stringify::writer::write_pair(
            &mut output,
            "name+role?",
            "value/with=reserved&stuff",
            false,
            &mut first_pair,
        );

        // Assert
        assert_eq!(output, "name%2Brole%3F=value%2Fwith%3Dreserved%26stuff");
        assert!(!first_pair);
        assert!(output.capacity() >= output.len());
    }
}
