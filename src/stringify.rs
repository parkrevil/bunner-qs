use crate::{QueryMap, StringifyError, StringifyOptions, StringifyResult};

pub fn stringify(map: &QueryMap) -> StringifyResult<String> {
    stringify_with_options(map, &StringifyOptions::default())
}

pub fn stringify_with_options(
    map: &QueryMap,
    options: &StringifyOptions,
) -> StringifyResult<String> {
    if map.is_empty() {
        return Ok(if options.add_query_prefix {
            String::from("?")
        } else {
            String::new()
        });
    }

    let mut pairs = Vec::with_capacity(map.len());

    for (key, values) in map.iter() {
        ensure_no_control(key).map_err(|_| StringifyError::InvalidKey { key: key.clone() })?;

        let encoded_key = encode_component(key, options.space_as_plus);

        if values.is_empty() {
            pairs.push(encoded_key.clone());
            continue;
        }

        for value in values {
            ensure_no_control(value)
                .map_err(|_| StringifyError::InvalidValue { key: key.clone() })?;
            let encoded_value = encode_component(value, options.space_as_plus);
            pairs.push(format!("{}={}", encoded_key, encoded_value));
        }
    }

    let mut body = pairs.join("&");

    if options.add_query_prefix {
        body.insert(0, '?');
    }

    Ok(body)
}

fn ensure_no_control(value: &str) -> Result<(), ()> {
    if value
        .chars()
        .any(|ch| matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}'))
    {
        Err(())
    } else {
        Ok(())
    }
}

fn encode_component(component: &str, space_as_plus: bool) -> String {
    if component.is_empty() {
        return String::new();
    }

    let mut encoded = String::with_capacity(component.len());

    for ch in component.chars() {
        if is_unreserved(ch) {
            encoded.push(ch);
        } else if ch == ' ' && space_as_plus {
            encoded.push('+');
        } else {
            let mut buffer = [0u8; 4];
            for byte in ch.encode_utf8(&mut buffer).as_bytes() {
                encoded.push('%');
                encoded.push(hex_digit(byte >> 4));
                encoded.push(hex_digit(byte & 0x0F));
            }
        }
    }

    encoded
}

fn is_unreserved(ch: char) -> bool {
    matches!(ch, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '.' | '_' | '~')
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + (value - 10)) as char,
        _ => unreachable!(),
    }
}
