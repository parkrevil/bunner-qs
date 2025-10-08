use bunner_qs_rs::stringify::api::stringify;
use bunner_qs_rs::{ParseOptions, StringifyOptions};
use serde_json::{Value, json};

pub const SIMPLE_TARGET_BYTES: usize = 500;
pub const SIMPLE_TOLERANCE: usize = 32;
pub const SIMPLE_MAX_DEPTH: usize = 2;

pub const MEDIUM_TARGET_BYTES: usize = 2 * 1024;
pub const MEDIUM_TOLERANCE: usize = 96;
pub const MEDIUM_MAX_DEPTH: usize = 4;

pub const HIGH_TARGET_BYTES: usize = 8 * 1024;
pub const HIGH_TOLERANCE: usize = 512;
pub const HIGH_MAX_DEPTH: usize = 8;

pub const EXTREME_TARGET_BYTES: usize = 16 * 1024;
pub const EXTREME_TOLERANCE: usize = 1024;
pub const EXTREME_MAX_DEPTH: usize = 16;

#[derive(Clone)]
pub struct Scenario {
    pub payload: Value,
    pub query: String,
    pub parse_options: ParseOptions,
    pub stringify_options: StringifyOptions,
    pub max_depth: usize,
}

pub type PayloadBuilder = fn(&str) -> Value;

pub fn scenario_simple() -> Scenario {
    let stringify_options = StringifyOptions::new().space_as_plus(false);
    stringify_options
        .validate()
        .expect("build stringify options");

    let (payload, query) = calibrate_payload(
        SIMPLE_TARGET_BYTES,
        SIMPLE_TOLERANCE,
        &stringify_options,
        SIMPLE_MAX_DEPTH,
        build_simple_payload,
    );

    let parse_options = parse_options_for(SIMPLE_TARGET_BYTES, SIMPLE_MAX_DEPTH);

    Scenario {
        payload,
        query,
        parse_options,
        stringify_options,
        max_depth: SIMPLE_MAX_DEPTH,
    }
}

pub fn scenario_medium() -> Scenario {
    let stringify_options = StringifyOptions::new().space_as_plus(false);
    stringify_options
        .validate()
        .expect("build stringify options");

    let (payload, query) = calibrate_payload(
        MEDIUM_TARGET_BYTES,
        MEDIUM_TOLERANCE,
        &stringify_options,
        MEDIUM_MAX_DEPTH,
        build_medium_payload,
    );

    let parse_options = parse_options_for(MEDIUM_TARGET_BYTES, MEDIUM_MAX_DEPTH);

    Scenario {
        payload,
        query,
        parse_options,
        stringify_options,
        max_depth: MEDIUM_MAX_DEPTH,
    }
}

pub fn scenario_high() -> Scenario {
    let stringify_options = StringifyOptions::new().space_as_plus(false);
    stringify_options
        .validate()
        .expect("build stringify options");

    let (payload, query) = calibrate_payload(
        HIGH_TARGET_BYTES,
        HIGH_TOLERANCE,
        &stringify_options,
        HIGH_MAX_DEPTH,
        build_high_payload,
    );

    let parse_options = parse_options_for(HIGH_TARGET_BYTES, HIGH_MAX_DEPTH);

    Scenario {
        payload,
        query,
        parse_options,
        stringify_options,
        max_depth: HIGH_MAX_DEPTH,
    }
}

pub fn scenario_extreme() -> Scenario {
    let stringify_options = StringifyOptions::new().space_as_plus(false);
    stringify_options
        .validate()
        .expect("build stringify options");

    let (payload, query) = calibrate_payload(
        EXTREME_TARGET_BYTES,
        EXTREME_TOLERANCE,
        &stringify_options,
        EXTREME_MAX_DEPTH,
        build_extreme_payload,
    );

    let parse_options = parse_options_for(EXTREME_TARGET_BYTES, EXTREME_MAX_DEPTH);

    Scenario {
        payload,
        query,
        parse_options,
        stringify_options,
        max_depth: EXTREME_MAX_DEPTH,
    }
}

pub fn calibrate_payload(
    target_len: usize,
    tolerance: usize,
    options: &StringifyOptions,
    expected_depth: usize,
    build_payload: PayloadBuilder,
) -> (Value, String) {
    let mut low = 1usize;
    let mut high = target_len * 2;
    let mut best_query = String::new();
    let mut best_payload = build_payload("");
    let mut best_diff = usize::MAX;

    while low <= high {
        let mid = (low + high) / 2;
        let filler = "x".repeat(mid);
        let candidate_payload = build_payload(&filler);
        let candidate_query =
            stringify(&candidate_payload, options).expect("stringify during calibration");
        let len = candidate_query.len();
        let depth = max_bracket_depth(&candidate_query);
        assert!(
            depth <= expected_depth,
            "scenario exceeded expected depth {} with depth {}",
            expected_depth,
            depth
        );

        if len >= target_len.saturating_sub(tolerance) && len <= target_len + tolerance {
            return (candidate_payload, candidate_query);
        }

        let diff = len.abs_diff(target_len);
        if diff < best_diff {
            best_diff = diff;
            best_query = candidate_query;
            best_payload = candidate_payload;
        }

        if len < target_len {
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }

    let len = best_query.len();
    assert!(
        len >= target_len.saturating_sub(tolerance) && len <= target_len + tolerance,
        "unable to calibrate query length within tolerance: got {} for target {}±{}",
        len,
        target_len,
        tolerance
    );

    (best_payload, best_query)
}

fn parse_options_for(target_len: usize, depth: usize) -> ParseOptions {
    let options = ParseOptions::new()
        .space_as_plus(false)
        .max_length(target_len + target_len / 4 + 512)
        .max_params(65_536)
        .max_depth(depth + 2);
    options.validate().expect("build parse options");
    options
}

fn build_simple_payload(filler: &str) -> Value {
    json!({
        "title": format!("simple_{:0>4}", filler.len()),
        "summary": filler,
        "flags": ["a", "b", substring_ascii(filler, 0, 6)],
        "aliases": [substring_ascii(filler, 4, 10)],
        "profile": {
            "name": "min",
            "team": "bench",
            "roles": ["lead"],
            "langs": ["rust"],
            "signature": substring_ascii(filler, 12, 10)
        },
        "regions": ["seoul", substring_ascii(filler, 22, 10)],
        "schedule": {
            "weekday": ["mon", substring_ascii(filler, 34, 8)]
        },
        "nested": build_nested_value(SIMPLE_MAX_DEPTH, filler),
    })
}

fn build_medium_payload(filler: &str) -> Value {
    let segments: Vec<String> = chunk_into_array(filler, 64);
    json!({
        "meta": {
            "kind": "medium",
            "checksum": format!("{:X}", filler.len() * 17),
            "segments": segments,
            "notes": {
                "summary": filler,
                "keywords": chunk_into_array(filler, 16)
            }
        },
        "records": [
            {
                "id": "r1",
                "attrs": {
                    "weights": ["0.1", "0.2", "0.3"],
                    "labels": ["alpha", "beta"],
                    "extra": [substring_ascii(filler, 10, 20)]
                }
            },
            {
                "id": "r2",
                "attrs": {
                    "weights": ["0.4", "0.5", "0.6"],
                    "labels": ["gamma", "delta"],
                    "extra": [substring_ascii(filler, 30, 20)]
                }
            }
        ],
        "windows": {
            "active": ["0-32", "32-64"],
            "archived": ["64-96"]
        },
        "nested": build_nested_value(MEDIUM_MAX_DEPTH, filler),
    })
}

fn build_high_payload(filler: &str) -> Value {
    let blocks: Vec<String> = chunk_into_array(filler, 128);
    let mut histories = Vec::new();
    for (i, block) in blocks.iter().enumerate() {
        histories.push(json!({
            "id": format!("h{i:04}"),
            "log": block,
            "events": [{"code": "evt", "payload": block.clone()}]
        }));
    }

    json!({
        "meta": {
            "kind": "high",
            "filler_len": filler.len().to_string(),
            "blocks": blocks,
            "alternatives": chunk_into_array(filler, 32),
        },
        "settings": {
            "modes": ["fast", "balanced", "accurate"],
            "thresholds": [{"name": "low", "value": "0.25"}, {"name": "high", "value": "0.75"}]
        },
        "histories": histories,
        "buffers": [{
            "name": "alpha",
            "frames": [[{"cell": "a0"}, {"cell": "a1"}]]
        }, {
            "name": "beta",
            "frames": [[{"cell": "b0"}]]
        }],
        "nested": build_nested_value(HIGH_MAX_DEPTH, filler),
    })
}

fn build_extreme_payload(filler: &str) -> Value {
    let chunks: Vec<String> = chunk_into_array(filler, 256);
    let mut batches = Vec::new();
    for (batch_idx, chunk) in chunks.iter().enumerate() {
        batches.push(json!({
            "batch": batch_idx.to_string(),
            "items": [
                {"idx": (batch_idx * 2).to_string(), "data": chunk},
                {"idx": (batch_idx * 2 + 1).to_string(), "data": chunk}
            ],
            "attributes": {
                "tags": chunk_into_array(chunk, 32),
                "extra": [{"label": "nested", "value": build_nested_value(4, chunk)}]
            }
        }));
    }

    json!({
        "meta": {
            "kind": "extreme",
            "chunks": chunks,
            "descriptor": filler
        },
        "payloads": batches,
        "matrix": {
            "layers": [[{"node": {"id": "n0"}}], [{"node": {"id": "n1"}}]]
        },
        "nested": build_nested_value(EXTREME_MAX_DEPTH, filler),
    })
}

fn build_nested_value(depth: usize, filler: &str) -> Value {
    if depth == 0 {
        return Value::String(substring_ascii(filler, 0, 16));
    }

    if depth % 2 == 0 {
        json!({ format!("lvl{depth:02}"): build_nested_value(depth - 1, filler) })
    } else {
        json!([build_nested_value(depth - 1, filler)])
    }
}

fn chunk_into_array(input: &str, chunk_size: usize) -> Vec<String> {
    let size = chunk_size.max(1);
    if input.is_empty() {
        return vec!["seed".into()];
    }
    input
        .as_bytes()
        .chunks(size)
        .map(|chunk| String::from_utf8(chunk.to_vec()).expect("valid utf8"))
        .collect()
}

fn substring_ascii(input: &str, start: usize, len: usize) -> String {
    if input.is_empty() {
        return String::new();
    }
    let s = start.min(input.len());
    let e = (s + len).min(input.len());
    input[s..e].to_string()
}

pub fn max_bracket_depth(query: &str) -> usize {
    query
        .split('&')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let key = segment.split_once('=').map(|(k, _)| k).unwrap_or(segment);
            bracket_depth_for_key(key)
        })
        .max()
        .unwrap_or(0)
}

fn bracket_depth_for_key(key: &str) -> usize {
    let mut depth = 0;
    let mut idx = 0;
    let bytes = key.as_bytes();

    while idx < bytes.len() {
        match bytes[idx] {
            b'[' => {
                depth += 1;
                idx += 1;
            }
            b'%' if idx + 2 < bytes.len() => {
                if let Some('[') = decode_percent(&bytes[idx + 1..idx + 3]) {
                    depth += 1;
                }
                idx += 3;
            }
            _ => {
                idx += 1;
            }
        }
    }

    depth
}

fn decode_percent(hex: &[u8]) -> Option<char> {
    fn from_hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }

    let hi = from_hex(*hex.first()?)?;
    let lo = from_hex(*hex.get(1)?)?;
    char::from_u32(((hi as u32) << 4) | (lo as u32))
}
