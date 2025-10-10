use crate::parsing::ParseError;
use crate::parsing::errors::ParseLocation;
use memchr::{memchr, memchr2};
use std::borrow::Cow;

pub(crate) fn decode_component<'a>(
    raw: &'a str,
    space_as_plus: bool,
    offset: usize,
    location: ParseLocation,
    scratch: &mut Vec<u8>,
) -> Result<Cow<'a, str>, ParseError> {
    if raw.is_empty() {
        return Ok(Cow::Borrowed(""));
    }

    let bytes = raw.as_bytes();
    let special_pos = if space_as_plus {
        memchr2(b'%', b'+', bytes)
    } else {
        memchr(b'%', bytes)
    };

    if special_pos.is_none() {
        return fast_path_ascii(raw, bytes, offset, location);
    }

    decode_with_special_chars(raw, bytes, space_as_plus, offset, location, scratch)
}

pub(crate) fn fast_path_ascii<'a>(
    raw: &'a str,
    bytes: &[u8],
    offset: usize,
    location: ParseLocation,
) -> Result<Cow<'a, str>, ParseError> {
    if let Some(idx) = bytes.iter().position(|&byte| byte <= 0x1F || byte == 0x7F) {
        return Err(ParseError::InvalidCharacter {
            character: bytes[idx] as char,
            index: offset + idx,
            location,
        });
    }
    Ok(Cow::Borrowed(raw))
}

pub(crate) fn decode_with_special_chars<'a>(
    raw: &'a str,
    bytes: &[u8],
    space_as_plus: bool,
    offset: usize,
    location: ParseLocation,
    scratch: &mut Vec<u8>,
) -> Result<Cow<'a, str>, ParseError> {
    scratch.clear();
    scratch.reserve(bytes.len());
    let mut modified = false;

    let mut cursor = 0usize;
    while cursor < bytes.len() {
        cursor = match bytes[cursor] {
            b'%' => {
                modified = true;
                decode_percent_sequence(bytes, cursor, offset, location, scratch)?
            }
            b'+' if space_as_plus => {
                modified = true;
                decode_plus(cursor, scratch)
            }
            byte if byte < 0x80 => {
                decode_ascii_run(bytes, cursor, offset, space_as_plus, location, scratch)?
            }
            _ => decode_utf8_cluster(raw, bytes, cursor, location, scratch)?,
        };
    }

    if !modified {
        scratch.clear();
        return Ok(Cow::Borrowed(raw));
    }

    finalize_decoded(location, scratch)
}

pub(crate) fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn decode_percent_sequence(
    bytes: &[u8],
    cursor: usize,
    offset: usize,
    location: ParseLocation,
    scratch: &mut Vec<u8>,
) -> Result<usize, ParseError> {
    if cursor + 2 >= bytes.len() {
        return Err(ParseError::InvalidPercentEncoding {
            index: offset + cursor,
            location,
        });
    }

    let hi = hex_value(bytes[cursor + 1]).ok_or(ParseError::InvalidPercentEncoding {
        index: offset + cursor,
        location,
    })?;
    let lo = hex_value(bytes[cursor + 2]).ok_or(ParseError::InvalidPercentEncoding {
        index: offset + cursor,
        location,
    })?;

    let decoded = (hi << 4) | lo;
    ensure_visible(decoded, offset + cursor, location)?;

    scratch.push(decoded);
    Ok(cursor + 3)
}

pub(crate) fn decode_plus(cursor: usize, scratch: &mut Vec<u8>) -> usize {
    scratch.push(b' ');
    cursor + 1
}

pub(crate) fn decode_ascii_run(
    bytes: &[u8],
    start: usize,
    offset: usize,
    space_as_plus: bool,
    location: ParseLocation,
    scratch: &mut Vec<u8>,
) -> Result<usize, ParseError> {
    ensure_visible(bytes[start], offset + start, location)?;

    let mut cursor = start + 1;
    while cursor < bytes.len() {
        let next = bytes[cursor];
        ensure_visible(next, offset + cursor, location)?;

        if next == b'%' || next >= 0x80 || (space_as_plus && next == b'+') {
            break;
        }

        cursor += 1;
    }

    scratch.extend_from_slice(&bytes[start..cursor]);
    Ok(cursor)
}

pub(crate) fn decode_utf8_cluster(
    raw: &str,
    bytes: &[u8],
    cursor: usize,
    location: ParseLocation,
    scratch: &mut Vec<u8>,
) -> Result<usize, ParseError> {
    let slice = &raw[cursor..];
    if let Some(ch) = slice.chars().next() {
        let len = ch.len_utf8();
        scratch.extend_from_slice(&bytes[cursor..cursor + len]);
        Ok(cursor + len)
    } else {
        Err(ParseError::InvalidUtf8 { location })
    }
}

pub(crate) fn finalize_decoded<'a>(
    location: ParseLocation,
    scratch: &mut Vec<u8>,
) -> Result<Cow<'a, str>, ParseError> {
    let decoded_len = scratch.len();
    let decoded_bytes = std::mem::take(scratch);

    match String::from_utf8(decoded_bytes) {
        Ok(decoded) => {
            scratch.reserve(decoded_len);
            Ok(Cow::Owned(decoded))
        }
        Err(err) => {
            *scratch = err.into_bytes();
            Err(ParseError::InvalidUtf8 { location })
        }
    }
}

pub(crate) fn ensure_visible(
    byte: u8,
    index: usize,
    location: ParseLocation,
) -> Result<(), ParseError> {
    if byte <= 0x1F || byte == 0x7F {
        Err(ParseError::InvalidCharacter {
            character: byte as char,
            index,
            location,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
pub(crate) use {
    decode_ascii_run as decode_ascii_run_for_test,
    decode_percent_sequence as decode_percent_sequence_for_test,
    decode_plus as decode_plus_for_test, decode_utf8_cluster as decode_utf8_cluster_for_test,
    decode_with_special_chars as decode_with_special_chars_for_test,
    ensure_visible as ensure_visible_for_test, fast_path_ascii as fast_path_ascii_for_test,
    finalize_decoded as finalize_decoded_for_test, hex_value as hex_value_for_test,
};

#[cfg(test)]
#[path = "decoder_test.rs"]
mod decoder_test;
