use crate::buffer_pool::acquire_bytes;
use crate::nested::{PatternState, insert_nested_value, parse_key_path};
use crate::value::QueryMap;
use crate::{ParseError, ParseOptions, ParseResult};

use serde::de::DeserializeOwned;

#[derive(Clone, Copy)]
struct ParseRuntime {
    space_as_plus: bool,
    max_params: Option<usize>,
    max_length: Option<usize>,
    max_depth: Option<usize>,
}

impl ParseRuntime {
    fn new(options: &ParseOptions) -> Self {
        Self {
            space_as_plus: options.space_as_plus,
            max_params: options.max_params,
            max_length: options.max_length,
            max_depth: options.max_depth,
        }
    }
}

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
    let runtime = ParseRuntime::new(options);
    let raw = input;

    if let Some(limit) = runtime.max_length
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
    let mut decode_scratch = acquire_bytes();
    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    let mut idx = 0usize;
    let mut segment_start = 0usize;
    let mut eq_index: Option<usize> = None;

    while idx <= len {
        let at_separator = idx == len || bytes[idx] == b'&';

        if at_separator {
            if idx > segment_start {
                pairs += 1;
                if let Some(limit) = runtime.max_params
                    && pairs > limit
                {
                    return Err(ParseError::TooManyParameters {
                        limit,
                        actual: pairs,
                    });
                }

                let (raw_key, raw_value) = match eq_index {
                    Some(eq_idx) => (&trimmed[segment_start..eq_idx], &trimmed[eq_idx + 1..idx]),
                    None => (&trimmed[segment_start..idx], ""),
                };

                let key_start = offset + segment_start;
                let key = decode_component(
                    raw_key,
                    runtime.space_as_plus,
                    key_start,
                    decode_scratch.as_mut(),
                )?;
                validate_brackets(&key, runtime.max_depth)?;

                let value_offset = match eq_index {
                    Some(eq_idx) => offset + eq_idx + 1,
                    None => offset + segment_start + raw_key.len(),
                };
                let value = decode_component(
                    raw_value,
                    runtime.space_as_plus,
                    value_offset,
                    decode_scratch.as_mut(),
                )?;

                let key_segments = parse_key_path(&key);
                insert_nested_value(&mut map, &key_segments, value, &mut pattern_state)?;
            }

            if idx == len {
                break;
            }

            idx += 1;
            segment_start = idx;
            eq_index = None;
            continue;
        }

        if bytes[idx] == b'=' && eq_index.is_none() {
            eq_index = Some(idx);
        }

        idx += 1;
    }
    Ok(map)
}

fn decode_component(
    raw: &str,
    space_as_plus: bool,
    offset: usize,
    scratch: &mut Vec<u8>,
) -> ParseResult<String> {
    if raw.is_empty() {
        return Ok(String::new());
    }

    let bytes = raw.as_bytes();
    let needs_percent = bytes.contains(&b'%');
    let needs_plus = space_as_plus && bytes.contains(&b'+');

    if !needs_percent && !needs_plus {
        validate_decoded(raw, offset)?;
        return Ok(raw.to_string());
    }

    scratch.clear();
    scratch.reserve(raw.len());

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
                scratch.push((hi << 4) | lo);
                cursor += 3;
            }
            b'+' if space_as_plus => {
                scratch.push(b' ');
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
                scratch.extend_from_slice(&bytes[cursor..cursor + len]);
                cursor += len;
            }
        }
    }

    let decoded = String::from_utf8(scratch.clone()).map_err(|_| ParseError::InvalidUtf8)?;
    scratch.clear();
    scratch.reserve(decoded.len());
    validate_decoded(&decoded, offset)?;
    Ok(decoded)
}

fn validate_decoded(decoded: &str, offset: usize) -> ParseResult<()> {
    if let Some(control) = decoded
        .chars()
        .find(|ch| matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}'))
    {
        return Err(ParseError::InvalidCharacter {
            character: control,
            index: offset,
        });
    }

    Ok(())
}

fn validate_brackets(key: &str, max_depth: Option<usize>) -> ParseResult<()> {
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

    if let Some(limit) = max_depth
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
