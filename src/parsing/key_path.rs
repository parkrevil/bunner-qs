use crate::parsing::ParseError;
use memchr::memchr_iter;

pub(crate) fn validate_brackets(key: &str, max_depth: Option<usize>) -> Result<(), ParseError> {
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
                        bracket: ']',
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
            bracket: '[',
        });
    }

    if let Some(limit) = max_depth
        && total_pairs > limit
    {
        return Err(ParseError::DepthExceeded {
            key: key.to_string(),
            limit,
            depth: total_pairs,
        });
    }

    Ok(())
}

pub(crate) fn duplicate_key_label(key: &str) -> String {
    key.to_string()
}

pub(crate) fn estimate_param_capacity(input: &str) -> usize {
    if input.is_empty() {
        return 0;
    }

    memchr_iter(b'&', input.as_bytes()).count() + 1
}

#[cfg(test)]
#[path = "key_path_test.rs"]
mod key_path_test;
