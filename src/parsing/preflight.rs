use crate::config::ParseOptions;
use crate::parsing::errors::ParseError;
use crate::util::is_ascii_control;

pub(crate) fn preflight<'a>(
    raw: &'a str,
    options: &ParseOptions,
) -> Result<(&'a str, usize), ParseError> {
    if let Some(limit) = options.max_length
        && raw.len() > limit
    {
        return Err(ParseError::InputTooLong { limit });
    }

    let (trimmed, offset) = match raw.strip_prefix('?') {
        Some(rest) => (rest, 1),
        None => (raw, 0),
    };

    for (idx, ch) in trimmed.char_indices() {
        check_character(ch, offset + idx)?;
    }

    Ok((trimmed, offset))
}

fn check_character(ch: char, index: usize) -> Result<(), ParseError> {
    if ch == '?' {
        return Err(ParseError::UnexpectedQuestionMark { index });
    }
    if is_disallowed_control(ch) {
        return Err(ParseError::InvalidCharacter {
            character: ch,
            index,
        });
    }
    Ok(())
}

fn is_disallowed_control(ch: char) -> bool {
    is_ascii_control(ch) || ch == ' '
}

#[cfg(test)]
#[path = "preflight_test.rs"]
mod preflight_test;
