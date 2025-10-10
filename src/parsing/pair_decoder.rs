use std::borrow::Cow;

use crate::config::ParseOptions;
use crate::parsing::ParseResult;
use crate::parsing::errors::ParseLocation;

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
    let key = decode_component(
        raw_key,
        options.space_as_plus,
        key_start,
        ParseLocation::Key,
        decode_scratch,
    )?;
    validate_brackets(key.as_ref(), options.max_depth)?;

    let value = decode_component(
        raw_value,
        options.space_as_plus,
        value_offset,
        ParseLocation::Value,
        decode_scratch,
    )?;

    Ok((key, value))
}

#[cfg(test)]
#[path = "pair_decoder_test.rs"]
mod pair_decoder_test;
