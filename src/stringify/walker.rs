use crate::model::Value;

pub(crate) struct StackItem<'a> {
    pub(crate) parent_len: usize,
    pub(crate) segment: Segment<'a>,
    pub(crate) value: &'a Value,
}

#[derive(Clone, Copy)]
pub(crate) enum Segment<'a> {
    Root(&'a str),
    Object(&'a str),
    Array(usize),
}

pub(crate) fn append_segment(buffer: &mut String, segment: Segment<'_>) {
    match segment {
        Segment::Root(key) => buffer.push_str(key),
        Segment::Object(sub_key) => {
            buffer.push('[');
            buffer.push_str(sub_key);
            buffer.push(']');
        }
        Segment::Array(index) => {
            buffer.push('[');
            push_usize_decimal(buffer, index);
            buffer.push(']');
        }
    }
}

fn push_usize_decimal(buffer: &mut String, mut value: usize) {
    if value == 0 {
        buffer.push('0');
        return;
    }

    const MAX_DIGITS: usize = 39; // Enough for 128-bit usize values
    let mut digits = [0u8; MAX_DIGITS];
    let mut pos = MAX_DIGITS;

    while value > 0 {
        pos -= 1;
        digits[pos] = b'0' + (value % 10) as u8;
        value /= 10;
    }

    let slice = &digits[pos..];
    // SAFETY: slice contains only ASCII digit bytes written above.
    buffer.push_str(unsafe { std::str::from_utf8_unchecked(slice) });
}

#[cfg(test)]
#[path = "walker_test.rs"]
mod walker_test;
