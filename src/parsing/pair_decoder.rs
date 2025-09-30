use std::borrow::Cow;

use crate::config::ParseOptions;
use crate::parsing::ParseResult;

use super::decoder::decode_component;
use super::key_path::validate_brackets;

pub(crate) fn decode_pair<'a>(
    raw_key: &'a str,
    raw_value: &'a str,
    key_start: usize,
    value_offset: usize,
    options: &ParseOptions,
    decode_scratch: &mut Vec<u8>,
) -> ParseResult<(Cow<'a, str>, Cow<'a, str>)> {
    let key = decode_component(raw_key, options.space_as_plus, key_start, decode_scratch)?;
    validate_brackets(key.as_ref(), options.max_depth)?;

    let value = decode_component(
        raw_value,
        options.space_as_plus,
        value_offset,
        decode_scratch,
    )?;

    Ok((key, value))
}
