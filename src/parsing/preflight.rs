use crate::error::ParseError;

use super::runtime::ParseRuntime;

pub(crate) fn preflight<'a>(
    raw: &'a str,
    runtime: &ParseRuntime,
) -> Result<(&'a str, usize), ParseError> {
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

fn is_disallowed_raw_char(ch: char) -> bool {
    matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}') || ch == ' '
}
