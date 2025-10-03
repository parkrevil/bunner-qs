use super::encode::{encode_key_into, encode_value_into, estimate_encoded_extra};

pub(crate) fn write_pair(
    output: &mut String,
    key: &str,
    value: &str,
    space_as_plus: bool,
    first_pair: &mut bool,
) {
    let separators = 1 + usize::from(!*first_pair);
    let base = separators + key.len() + value.len();
    let extra = estimate_encoded_extra(key, space_as_plus)
        .saturating_add(estimate_encoded_extra(value, space_as_plus));
    let required = base.saturating_add(extra);
    let available = output.capacity() - output.len();
    if available < required {
        output.reserve(required - available);
    }

    if !*first_pair {
        output.push('&');
    } else {
        *first_pair = false;
    }

    encode_key_into(output, key, space_as_plus);
    output.push('=');
    encode_value_into(output, value, space_as_plus);
}

#[cfg(test)]
#[path = "writer_test.rs"]
mod writer_test;
