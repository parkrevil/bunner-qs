use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use std::fmt::Write as _;

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

pub(crate) fn encode_key_into(buffer: &mut String, key: &str, space_as_plus: bool) {
    encode_into(key, space_as_plus, buffer);
}

pub(crate) fn encode_value_into(buffer: &mut String, value: &str, space_as_plus: bool) {
    encode_into(value, space_as_plus, buffer);
}

pub(crate) fn estimate_encoded_extra(component: &str, space_as_plus: bool) -> usize {
    component
        .bytes()
        .filter(|byte| needs_encoding(*byte, space_as_plus))
        .count()
        .saturating_mul(2)
}

fn encode_into(component: &str, space_as_plus: bool, buffer: &mut String) {
    if component.is_empty() {
        return;
    }

    if !space_as_plus {
        append_encoded(component, buffer);
        return;
    }

    let mut tail = 0;
    for (idx, ch) in component.char_indices() {
        if ch == ' ' {
            if tail < idx {
                append_encoded(&component[tail..idx], buffer);
            }
            buffer.push('+');
            tail = idx + ch.len_utf8();
        }
    }

    if tail < component.len() {
        append_encoded(&component[tail..], buffer);
    }
}

fn append_encoded(segment: &str, buffer: &mut String) {
    if segment.is_empty() {
        return;
    }

    let _ = write!(
        buffer,
        "{}",
        utf8_percent_encode(segment, COMPONENT_ENCODE_SET)
    );
}

#[inline]
fn needs_encoding(byte: u8, space_as_plus: bool) -> bool {
    if byte >= 0x80 {
        return true;
    }

    match byte {
        b' ' => !space_as_plus,
        0x00..=0x1F | 0x7F => true,
        b'"' | b'#' | b'%' | b'&' | b'+' | b',' | b'/' | b':' | b';' | b'<' | b'>' | b'='
        | b'?' | b'@' | b'[' | b'\\' | b']' | b'^' | b'`' | b'{' | b'|' | b'}' => true,
        _ => false,
    }
}

#[cfg(test)]
#[path = "encode_test.rs"]
mod encode_test;
