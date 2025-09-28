use crate::parsing::ParseError;
use memchr::memchr_iter;

use super::runtime::ParseRuntime;

pub(crate) fn validate_brackets(
    key: &str,
    max_depth: Option<usize>,
    diagnostics: bool,
) -> Result<(), ParseError> {
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

pub(crate) fn duplicate_key_label(runtime: &ParseRuntime, key: &str) -> String {
    duplicate_key_for_diagnostics(key, runtime.diagnostics)
}

pub(crate) fn duplicate_key_for_diagnostics(key: &str, diagnostics: bool) -> String {
    if diagnostics {
        key.to_string()
    } else {
        key.split('[').next().unwrap_or(key).to_string()
    }
}

pub(crate) fn estimate_param_capacity(input: &str) -> usize {
    if input.is_empty() {
        return 0;
    }

    memchr_iter(b'&', input.as_bytes()).count() + 1
}
