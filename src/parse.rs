use crate::nested::{PatternState, insert_nested_value, parse_key_path};
use crate::value::QueryMap;
use crate::{ParseError, ParseOptions, ParseResult};

use serde::de::DeserializeOwned;

pub fn parse<T>(input: impl AsRef<str>) -> ParseResult<T>
where
    T: DeserializeOwned + Default,
{
    parse_with(input, &ParseOptions::default())
}

pub fn parse_with<T>(input: impl AsRef<str>, options: &ParseOptions) -> ParseResult<T>
where
    T: DeserializeOwned + Default,
{
    let map = parse_query_map(input.as_ref(), options)?;
    if map.is_empty() {
        Ok(T::default())
    } else {
        map.to_struct::<T>().map_err(ParseError::from)
    }
}

pub(crate) fn parse_query_map(input: &str, options: &ParseOptions) -> ParseResult<QueryMap> {
    let raw = input;

    if let Some(limit) = options.max_length
        && raw.len() > limit
    {
        return Err(ParseError::InputTooLong { limit });
    }

    let (trimmed, offset) = match raw.strip_prefix('?') {
        Some(rest) => (rest, 1),
        None => (raw, 0),
    };

    if trimmed.is_empty() {
        return Ok(QueryMap::new());
    }

    for (idx, ch) in trimmed.char_indices() {
        if ch == '?' {
            return Err(ParseError::UnexpectedQuestionMark {
                index: offset + idx,
            });
        }
        if is_disallowed_raw_char(ch) {
            return Err(ParseError::InvalidCharacter {
                character: ch,
                index: offset + idx,
            });
        }
    }

    let mut map = QueryMap::new();
    let mut pattern_state = PatternState::default();
    let mut pairs = 0usize;
    let mut start = 0;
    let len = trimmed.len();

    while start <= len {
        let end = match trimmed[start..].find('&') {
            Some(pos) => start + pos,
            None => len,
        };
        let segment = &trimmed[start..end];

        if !segment.is_empty() {
            pairs += 1;
            if let Some(limit) = options.max_params
                && pairs > limit
            {
                return Err(ParseError::TooManyParameters {
                    limit,
                    actual: pairs,
                });
            }

            let eq_index = segment.find('=');
            let (raw_key, raw_value) = match eq_index {
                Some(idx) => (&segment[..idx], &segment[idx + 1..]),
                None => (segment, ""),
            };

            let key_start = offset + start;
            let key = decode_component(raw_key, options, key_start)?;
            validate_brackets(&key, options)?;

            let value_offset = match eq_index {
                Some(idx) => offset + start + idx + 1,
                None => offset + start + raw_key.len(),
            };
            let value = decode_component(raw_value, options, value_offset)?;

            let key_segments = parse_key_path(&key);
            insert_nested_value(&mut map, &key_segments, value, &mut pattern_state)?;
        }

        if end == len {
            break;
        }

        start = end + 1;
    }

    Ok(map)
}

fn decode_component(raw: &str, options: &ParseOptions, offset: usize) -> ParseResult<String> {
    if raw.is_empty() {
        return Ok(String::new());
    }

    let bytes = raw.as_bytes();
    let mut cursor = 0usize;
    let mut decoded = Vec::with_capacity(raw.len());

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
                decoded.push((hi << 4) | lo);
                cursor += 3;
            }
            b'+' if options.space_as_plus => {
                decoded.push(b' ');
                cursor += 1;
            }
            byte => {
                if byte <= 0x20 || byte == 0x7F {
                    return Err(ParseError::InvalidCharacter {
                        character: byte as char,
                        index: offset + cursor,
                    });
                }
                let slice = &raw[cursor..];
                let ch = slice.chars().next().unwrap();
                let len = ch.len_utf8();
                decoded.extend_from_slice(&bytes[cursor..cursor + len]);
                cursor += len;
            }
        }
    }

    let result = String::from_utf8(decoded).map_err(|_| ParseError::InvalidUtf8)?;

    if let Some(control) = result
        .chars()
        .find(|ch| matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}'))
    {
        return Err(ParseError::InvalidCharacter {
            character: control,
            index: offset,
        });
    }

    Ok(result)
}

fn validate_brackets(key: &str, options: &ParseOptions) -> ParseResult<()> {
    let mut open = 0usize;
    let mut total_pairs = 0usize;

    for ch in key.chars() {
        match ch {
            '[' => {
                open += 1;
                total_pairs += 1;
            }
            ']' => {
                if open == 0 {
                    return Err(ParseError::UnmatchedBracket {
                        key: key.to_string(),
                    });
                }
                open -= 1;
            }
            _ => {}
        }
    }

    if open != 0 {
        return Err(ParseError::UnmatchedBracket {
            key: key.to_string(),
        });
    }

    if let Some(limit) = options.max_depth
        && total_pairs > limit
    {
        return Err(ParseError::DepthExceeded {
            key: key.to_string(),
            limit,
        });
    }

    Ok(())
}

fn is_disallowed_raw_char(ch: char) -> bool {
    matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}') || ch == ' '
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
