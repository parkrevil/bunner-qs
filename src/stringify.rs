use crate::{QueryMap, StringifyError, StringifyOptions, StringifyResult, Value};

pub fn stringify(map: &QueryMap) -> StringifyResult<String> {
    stringify_with(map, StringifyOptions::default())
}

pub fn stringify_with(map: &QueryMap, options: StringifyOptions) -> StringifyResult<String> {
    if map.is_empty() {
        return Ok(if options.add_query_prefix {
            String::from("?")
        } else {
            String::new()
        });
    }

    let mut pairs = Vec::new();

    for (key, value) in map.iter() {
        ensure_no_control(key).map_err(|_| StringifyError::InvalidKey { key: key.clone() })?;

        flatten_value(key, value, &mut pairs, options.space_as_plus)?;
    }

    let mut body = pairs.join("&");

    if options.add_query_prefix {
        body.insert(0, '?');
    }

    Ok(body)
}

fn flatten_value(
    base_key: &str,
    value: &Value,
    pairs: &mut Vec<String>,
    space_as_plus: bool,
) -> StringifyResult<()> {
    match value {
        Value::String(s) => {
            ensure_no_control(s).map_err(|_| StringifyError::InvalidValue {
                key: base_key.to_string(),
            })?;
            let encoded_key = encode_component(base_key, space_as_plus);
            let encoded_value = encode_component(s, space_as_plus);
            pairs.push(format!("{}={}", encoded_key, encoded_value));
        }
        Value::Array(arr) => {
            for (idx, item) in arr.iter().enumerate() {
                let key = format!("{}[{}]", base_key, idx);
                flatten_value(&key, item, pairs, space_as_plus)?;
            }
        }
        Value::Object(obj) => {
            for (sub_key, sub_value) in obj.iter() {
                let key = format!("{}[{}]", base_key, sub_key);
                flatten_value(&key, sub_value, pairs, space_as_plus)?;
            }
        }
    }
    Ok(())
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
