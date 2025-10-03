use memchr::memchr;
use smallvec::SmallVec;

pub fn parse_key_path(key: &str) -> SmallVec<[&str; 16]> {
    let mut segments: SmallVec<[&str; 16]> = SmallVec::new();
    let bytes = key.as_bytes();
    let mut start = 0usize;
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        if bytes[cursor] == b'[' {
            if start < cursor {
                segments.push(&key[start..cursor]);
            }
            cursor += 1;
            start = cursor;
            if cursor >= bytes.len() {
                break;
            }

            let rel = memchr(b']', &bytes[cursor..]);
            let end = rel.map(|offset| cursor + offset).unwrap_or(bytes.len());
            if start < end {
                segments.push(&key[start..end]);
            } else {
                segments.push("");
            }
            cursor = end.saturating_add(1);
            start = cursor;
        } else {
            cursor += 1;
        }
    }

    if start < key.len() {
        segments.push(&key[start..]);
    }

    segments
}

#[cfg(test)]
#[path = "key_path_test.rs"]
mod key_path_test;
