use super::encode::{encode_key_into, encode_value_into};

pub(crate) fn write_pair(
    output: &mut String,
    key: &str,
    value: &str,
    space_as_plus: bool,
    first_pair: &mut bool,
) {
    let base_len = key.len() + value.len();
    let separators = if *first_pair { 1 } else { 2 };
    let available = output.capacity() - output.len();
    let conservative_need = separators + base_len;
    if available < conservative_need {
        let worst_case = separators + base_len.saturating_mul(3);
        output.reserve(worst_case - available);
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
