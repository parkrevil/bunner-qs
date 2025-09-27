use crate::arena::{ArenaQueryMap, ArenaValue, ParseArena, ParseArenaGuard, acquire_parse_arena};
use crate::buffer_pool::acquire_bytes;
use crate::nested::{PatternState, acquire_pattern_state, insert_nested_value_arena, parse_key_path};
use crate::options::{ParseOptions, global_parse_diagnostics, global_serde_fastpath};
use crate::serde_bridge::{arena_map_to_json_value, from_arena_query_map};
use crate::value::QueryMap;
use crate::{ParseError, ParseResult};
use ahash::AHashSet;
use memchr::{memchr, memchr_iter};
use serde::de::DeserializeOwned;
use serde_json::Value as JsonValue;
use std::any::TypeId;
use std::borrow::Cow;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

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
    T: DeserializeOwned + Default + 'static,
{
    parse_with(input, &ParseOptions::default())
}

pub fn parse_with<T>(input: impl AsRef<str>, options: &ParseOptions) -> ParseResult<T>
where
    T: DeserializeOwned + Default + 'static,
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

    with_arena_query_map(trimmed, offset, &runtime, |_, arena_map| {
        if arena_map.len() == 0 {
            Ok(T::default())
        } else {
            if runtime.serde_fastpath && TypeId::of::<T>() == TypeId::of::<JsonValue>() {
                let json_value = arena_map_to_json_value(arena_map);
                let json_value = ManuallyDrop::new(json_value);
                let ptr = (&*json_value) as *const JsonValue as *const T;
                // SAFETY: TypeId equality guarantees T is exactly JsonValue.
                let value = unsafe { ptr.read() };
                return Ok(value);
            }
            from_arena_query_map::<T>(arena_map).map_err(ParseError::from)
        }
    })
}

#[allow(dead_code)]
pub(crate) fn parse_query_map(input: &str, options: &ParseOptions) -> ParseResult<QueryMap> {
    let runtime = ParseRuntime::new(options);
    let (trimmed, offset) = preflight(input, &runtime)?;

    if trimmed.is_empty() {
        return Ok(QueryMap::new());
    }

    with_arena_query_map(trimmed, offset, &runtime, |_, arena_map| {
        Ok(arena_map.to_owned())
    })
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
    let needs_percent = memchr(b'%', bytes).is_some();
    let needs_plus = space_as_plus && memchr(b'+', bytes).is_some();

    if !needs_percent && !needs_plus {
        if let Some(idx) = bytes
            .iter()
            .position(|&byte| byte <= 0x1F || byte == 0x7F)
        {
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
                        if next == b'%' || (next == b'+' && space_as_plus) {
                            break;
                        }
                        if next <= 0x1F || next == 0x7F || next >= 0x80 {
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

const ARENA_REUSE_UPPER_BOUND: usize = 32 * 1024;

fn with_arena_query_map<R, F>(
    trimmed: &str,
    offset: usize,
    runtime: &ParseRuntime,
    finalize: F,
) -> ParseResult<R>
where
    F: for<'arena> FnOnce(&'arena ParseArena, &ArenaQueryMap<'arena>) -> ParseResult<R>,
{
    let arena_capacity = trimmed.len().saturating_mul(2);
    let arena_lease = if arena_capacity <= ARENA_REUSE_UPPER_BOUND {
        ArenaLease::Guard(acquire_parse_arena(arena_capacity))
    } else {
        ArenaLease::Owned(ParseArena::with_capacity(arena_capacity))
    };
    let arena: &ParseArena = &arena_lease;
    let estimated_pairs = estimate_param_capacity(trimmed);
    let mut arena_map = ArenaQueryMap::with_capacity(arena, estimated_pairs);
    let mut pattern_state = acquire_pattern_state();
    let mut pairs = 0usize;
    let mut decode_scratch = acquire_bytes();
    let bytes = trimmed.as_bytes();
    let mut cursor = 0usize;

    for segment_end in memchr_iter(b'&', bytes).chain(std::iter::once(bytes.len())) {
        if segment_end > cursor {
            pairs += 1;
            if let Some(limit) = runtime.max_params
                && pairs > limit
            {
                return Err(ParseError::TooManyParameters {
                    limit,
                    actual: pairs,
                });
            }

            let eq_relative = memchr(b'=', &bytes[cursor..segment_end]);
            let eq_index = eq_relative.map(|rel| cursor + rel);

            let raw_key_end = eq_index.unwrap_or(segment_end);
            let raw_key = &trimmed[cursor..raw_key_end];
            let raw_value = eq_index
                .map(|idx| &trimmed[idx + 1..segment_end])
                .unwrap_or("");

            let key_start = offset + cursor;
            let key = decode_component(
                raw_key,
                runtime.space_as_plus,
                key_start,
                decode_scratch.as_mut(),
            )?;
            validate_brackets(key.as_ref(), runtime.max_depth, runtime.diagnostics)?;

            let value_offset = eq_index
                .map(|idx| offset + idx + 1)
                .unwrap_or(offset + cursor + raw_key.len());
            let value = decode_component(
                raw_value,
                runtime.space_as_plus,
                value_offset,
                decode_scratch.as_mut(),
            )?;

            insert_pair_arena(
                arena,
                &mut arena_map,
                &mut pattern_state,
                runtime,
                key,
                value,
            )?;
        }

        cursor = segment_end.saturating_add(1);
    }

    finalize(arena, &arena_map)
}

enum ArenaLease {
    Guard(ParseArenaGuard),
    Owned(ParseArena),
}

impl Deref for ArenaLease {
    type Target = ParseArena;

    fn deref(&self) -> &Self::Target {
        match self {
            ArenaLease::Guard(guard) => guard,
            ArenaLease::Owned(arena) => arena,
        }
    }
}

impl DerefMut for ArenaLease {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ArenaLease::Guard(guard) => guard,
            ArenaLease::Owned(arena) => arena,
        }
    }
}

fn insert_pair_arena<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    pattern_state: &mut PatternState,
    runtime: &ParseRuntime,
    key: Cow<'_, str>,
    value: Cow<'_, str>,
) -> ParseResult<()> {
    if !key.is_empty() && !key.contains('[') {
        let key_str = key.as_ref();
        let value_ref = arena.alloc_str(value.as_ref());
        map.try_insert_str(arena, key_str, ArenaValue::string(value_ref))
            .map_err(|_| ParseError::DuplicateKey {
                key: duplicate_key_label(runtime, key_str),
            })?;
        return Ok(());
    }

    let key_segments = parse_key_path(key.as_ref());
    let value_ref = arena.alloc_str(value.as_ref());
    insert_nested_value_arena(
        arena,
        map,
        &key_segments,
        value_ref,
        pattern_state,
        runtime.diagnostics,
    )
}

fn estimate_param_capacity(input: &str) -> usize {
    if input.is_empty() {
        return 0;
    }

    memchr_iter(b'&', input.as_bytes()).count() + 1
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
    if bytes.iter().any(|b| matches!(b, b'[' | b']' | b'%')) {
        return None;
    }

    if !runtime.space_as_plus && bytes.contains(&b'+') {
        return None;
    }

    let mut pairs = 0usize;
    let mut seen: AHashSet<&str> = AHashSet::with_capacity(estimate_param_capacity(trimmed));

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
