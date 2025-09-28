use crate::parsing::ParseError;
use memchr::{memchr, memchr2};
use std::borrow::Cow;

pub(crate) fn decode_component<'a>(
    raw: &'a str,
    space_as_plus: bool,
    offset: usize,
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
        if let Some(idx) = bytes.iter().position(|&byte| byte <= 0x1F || byte == 0x7F) {
            return Err(ParseError::InvalidCharacter {
                character: bytes[idx] as char,
                index: offset + idx,
            });
        }
        return Ok(Cow::Borrowed(raw));
    }

    scratch.clear();
    scratch.reserve(bytes.len());

    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'%' => {
                if cursor + 2 >= bytes.len() {
                    return Err(ParseError::InvalidPercentEncoding {
                        index: offset + cursor,
                    });
                }
                let hi =
                    hex_value(bytes[cursor + 1]).ok_or(ParseError::InvalidPercentEncoding {
                        index: offset + cursor,
                    })?;
                let lo =
                    hex_value(bytes[cursor + 2]).ok_or(ParseError::InvalidPercentEncoding {
                        index: offset + cursor,
                    })?;
                let decoded = (hi << 4) | lo;
                if decoded <= 0x1F || decoded == 0x7F {
                    return Err(ParseError::InvalidCharacter {
                        character: decoded as char,
                        index: offset + cursor,
                    });
                }
                scratch.push(decoded);
                cursor += 3;
            }
            b'+' if space_as_plus => {
                scratch.push(b' ');
                cursor += 1;
            }
            byte => {
                if byte <= 0x1F || byte == 0x7F {
                    return Err(ParseError::InvalidCharacter {
                        character: byte as char,
                        index: offset + cursor,
                    });
                }

                if byte < 0x80 {
                    let start = cursor;
                    cursor += 1;

                    while cursor < bytes.len() {
                        let next = bytes[cursor];

                        if next <= 0x1F || next == 0x7F {
                            return Err(ParseError::InvalidCharacter {
                                character: next as char,
                                index: offset + cursor,
                            });
                        }

                        if next == b'%' || next >= 0x80 || (space_as_plus && next == b'+') {
                            break;
                        }

                        cursor += 1;
                    }

                    scratch.extend_from_slice(&bytes[start..cursor]);
                } else {
                    let slice = &raw[cursor..];
                    let ch = slice.chars().next().unwrap();
                    let len = ch.len_utf8();
                    scratch.extend_from_slice(&bytes[cursor..cursor + len]);
                    cursor += len;
                }
            }
        }
    }

    let decoded_len = scratch.len();
    let decoded_bytes = std::mem::take(scratch);
    match String::from_utf8(decoded_bytes) {
        Ok(decoded) => {
            scratch.reserve(decoded_len);
            Ok(Cow::Owned(decoded))
        }
        Err(err) => {
            *scratch = err.into_bytes();
            Err(ParseError::InvalidUtf8)
        }
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
