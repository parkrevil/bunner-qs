use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

const fn build_component_set() -> AsciiSet {
    CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'#')
        .add(b'%')
        .add(b'&')
        .add(b'+')
        .add(b',')
        .add(b'/')
        .add(b':')
        .add(b';')
        .add(b'<')
        .add(b'>')
        .add(b'=')
        .add(b'?')
        .add(b'@')
        .add(b'[')
        .add(b'\\')
        .add(b']')
        .add(b'^')
        .add(b'`')
        .add(b'{')
        .add(b'|')
        .add(b'}')
}

const COMPONENT_ENCODE_SET: &AsciiSet = &build_component_set();

fn encode_with_set(component: &str, space_as_plus: bool) -> String {
    if component.is_empty() {
        return String::new();
    }

    let mut encoded = String::with_capacity(component.len());
    let mut buffer = [0u8; 4];

    for ch in component.chars() {
        if ch == ' ' && space_as_plus {
            encoded.push('+');
            continue;
        }

        let slice = ch.encode_utf8(&mut buffer);
        encoded.push_str(&utf8_percent_encode(slice, COMPONENT_ENCODE_SET).to_string());
    }

    encoded
}

pub fn encode_key(key: &str, space_as_plus: bool) -> String {
    encode_with_set(key, space_as_plus)
}

pub fn encode_value(value: &str, space_as_plus: bool) -> String {
    encode_with_set(value, space_as_plus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_reserved_characters() {
        let encoded = encode_value("a&b=c", false);
        assert_eq!(encoded, "a%26b%3Dc");
    }

    #[test]
    fn encodes_space_as_plus_when_requested() {
        let encoded = encode_value("hello world", true);
        assert_eq!(encoded, "hello+world");
    }

    #[test]
    fn leaves_unreserved_characters_intact() {
        let original = "abc-_.~";
        let encoded = encode_key(original, false);
        assert_eq!(encoded, original);
    }
}
