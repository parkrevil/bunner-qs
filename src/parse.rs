use crate::buffer_pool::acquire_bytes;
use crate::nested::{PatternState, insert_nested_value, parse_key_path};
use crate::options::{ParseOptions, global_parse_diagnostics, global_serde_fastpath};
use crate::value::{QueryMap, Value};
use crate::{ParseError, ParseResult};

use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Clone, Copy)]
struct ParseRuntime {
    space_as_plus: bool,
    max_params: Option<usize>,
    max_length: Option<usize>,
    max_depth: Option<usize>,
    diagnostics: bool,
    serde_fastpath: bool,
}

impl ParseRuntime {
    fn new(options: &ParseOptions) -> Self {
        Self {
            space_as_plus: options.space_as_plus,
            max_params: options.max_params,
            max_length: options.max_length,
            max_depth: options.max_depth,
            diagnostics: global_parse_diagnostics(),
            serde_fastpath: global_serde_fastpath(),
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
    let raw = input.as_ref();
    let runtime = ParseRuntime::new(options);
    let (trimmed, offset) = preflight(raw, &runtime)?;

    if trimmed.is_empty() {
        return Ok(T::default());
    }

    if let Some(result) = try_parse_direct(trimmed, &runtime) {
        return result;
    }

    let map = parse_query_map_impl(trimmed, offset, &runtime)?;
    if map.is_empty() {
        Ok(T::default())
    } else {
        map.to_struct::<T>().map_err(ParseError::from)
    }
}

#[allow(dead_code)]
pub(crate) fn parse_query_map(input: &str, options: &ParseOptions) -> ParseResult<QueryMap> {
    let runtime = ParseRuntime::new(options);
    let (trimmed, offset) = preflight(input, &runtime)?;

    if trimmed.is_empty() {
        return Ok(QueryMap::new());
    }

    parse_query_map_impl(trimmed, offset, &runtime)
}

fn decode_component<'a>(
    raw: &'a str,
    space_as_plus: bool,
    offset: usize,
    scratch: &mut Vec<u8>,
) -> ParseResult<Cow<'a, str>> {
    if raw.is_empty() {
        return Ok(Cow::Borrowed(""));
    }

    let bytes = raw.as_bytes();
    let needs_percent = bytes.contains(&b'%');
    let needs_plus = space_as_plus && bytes.contains(&b'+');

    if !needs_percent && !needs_plus {
        validate_decoded(raw, offset)?;
        return Ok(Cow::Borrowed(raw));
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
                if byte < 0x80 {
                    let start = cursor;
                    cursor += 1;
                    while cursor < bytes.len() {
                        let next = bytes[cursor];
                        if next == b'%' || (next == b'+' && space_as_plus) {
                            break;
                        }
                        if next <= 0x20 || next == 0x7F || next >= 0x80 {
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

    let decoded_bytes = std::mem::take(scratch);
    let decoded = String::from_utf8(decoded_bytes).map_err(|_| ParseError::InvalidUtf8)?;
    scratch.reserve(decoded.len());
    validate_decoded(&decoded, offset)?;
    Ok(Cow::Owned(decoded))
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

fn validate_brackets(key: &str, max_depth: Option<usize>, diagnostics: bool) -> ParseResult<()> {
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
                        key: duplicate_key_for_diagnostics(key, diagnostics),
                    });
                }
                open -= 1;
            }
            _ => {}
        }
    }

    if open != 0 {
        return Err(ParseError::UnmatchedBracket {
            key: duplicate_key_for_diagnostics(key, diagnostics),
        });
    }

    if let Some(limit) = max_depth
        && total_pairs > limit
    {
        return Err(ParseError::DepthExceeded {
            key: duplicate_key_for_diagnostics(key, diagnostics),
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

fn preflight<'a>(raw: &'a str, runtime: &ParseRuntime) -> ParseResult<(&'a str, usize)> {
    if let Some(limit) = runtime.max_length
        && raw.len() > limit
    {
        return Err(ParseError::InputTooLong { limit });
    }

    let (trimmed, offset) = match raw.strip_prefix('?') {
        Some(rest) => (rest, 1),
        None => (raw, 0),
    };

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

    Ok((trimmed, offset))
}

fn parse_query_map_impl(
    trimmed: &str,
    offset: usize,
    runtime: &ParseRuntime,
) -> ParseResult<QueryMap> {
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
                validate_brackets(key.as_ref(), runtime.max_depth, runtime.diagnostics)?;

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

                if !key.is_empty() && !key.contains('[') {
                    if map.contains_key(key.as_ref()) {
                        let label = duplicate_key_label(runtime, key.as_ref());
                        return Err(ParseError::DuplicateKey { key: label });
                    }
                    map.insert(key.into_owned(), Value::String(value.into_owned()));
                } else {
                    let key_segments = parse_key_path(key.as_ref());
                    insert_nested_value(
                        &mut map,
                        &key_segments,
                        value.into_owned(),
                        &mut pattern_state,
                        runtime.diagnostics,
                    )?;
                }
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

fn duplicate_key_label(runtime: &ParseRuntime, key: &str) -> String {
    duplicate_key_for_diagnostics(key, runtime.diagnostics)
}

fn duplicate_key_for_diagnostics(key: &str, diagnostics: bool) -> String {
    if diagnostics {
        key.to_string()
    } else {
        key.split('[').next().unwrap_or(key).to_string()
    }
}

fn try_parse_direct<T>(trimmed: &str, runtime: &ParseRuntime) -> Option<ParseResult<T>>
where
    T: DeserializeOwned + Default,
{
    if !runtime.serde_fastpath {
        return None;
    }

    let bytes = trimmed.as_bytes();
    if bytes.iter().any(|b| matches!(b, b'[' | b']' | b'%' | b'+')) {
        return None;
    }

    let mut pairs = 0usize;
    let mut seen = HashSet::new();

    for segment in trimmed.split('&') {
        if segment.is_empty() {
            return None;
        }

        pairs += 1;
        if let Some(limit) = runtime.max_params
            && pairs > limit
        {
            return Some(Err(ParseError::TooManyParameters {
                limit,
                actual: pairs,
            }));
        }

        let (key, _) = match segment.split_once('=') {
            Some((key, value)) => (key, value),
            None => (segment, ""),
        };

        if key.is_empty() {
            return None;
        }

        if !seen.insert(key) {
            return Some(Err(ParseError::DuplicateKey {
                key: key.to_string(),
            }));
        }
    }

    match serde_urlencoded::from_str::<T>(trimmed) {
        Ok(value) => Some(Ok(value)),
        Err(_) => None,
    }
}
