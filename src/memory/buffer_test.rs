use super::{MAX_BYTE_BUFFER_CAPACITY, MAX_STRING_BUFFER_CAPACITY, acquire_bytes, acquire_string};

fn record_string_capacity<F>(mut fill: F) -> usize
where
    F: FnMut(&mut String),
{
    let mut guard = acquire_string();
    let buf = guard.as_mut();
    fill(buf);
    buf.capacity()
}

fn assert_string_reuse(buf: &String, min_capacity: usize) {
    assert_eq!(buf.len(), 0, "reused buffer should be cleared");
    assert!(
        buf.capacity() >= min_capacity,
        "buffer should retain at least prior capacity"
    );
}

fn assert_string_dropped(buf: &String, max_capacity: usize) {
    assert!(
        buf.capacity() <= max_capacity,
        "oversized buffers should not be reused"
    );
    assert_eq!(buf.len(), 0);
}

fn record_byte_capacity<F>(mut fill: F) -> usize
where
    F: FnMut(&mut Vec<u8>),
{
    let mut guard = acquire_bytes();
    let buf = guard.as_mut();
    fill(buf);
    buf.capacity()
}

fn assert_byte_reuse(buf: &Vec<u8>, min_capacity: usize) {
    assert_eq!(buf.len(), 0, "byte buffer should be cleared on reuse");
    assert!(
        buf.capacity() >= min_capacity,
        "byte buffer should retain previous capacity"
    );
}

fn assert_byte_dropped(buf: &Vec<u8>, max_capacity: usize) {
    assert!(
        buf.capacity() <= max_capacity,
        "oversized byte buffers should not be reused"
    );
    assert_eq!(buf.len(), 0);
}

mod acquire_string {
    use super::*;

    #[test]
    fn should_reuse_string_buffer_when_capacity_is_preserved_then_retain_previous_capacity() {
        let recorded_capacity = record_string_capacity(|buf| buf.push_str("hello world"));

        let mut guard = acquire_string();
        let buf = guard.as_mut();

        assert_string_reuse(buf, recorded_capacity);
    }

    #[test]
    fn should_drop_string_buffer_when_capacity_exceeds_limit_then_release_oversized_buffer() {
        let oversized = MAX_STRING_BUFFER_CAPACITY + 1024;
        record_string_capacity(|buf| {
            buf.reserve_exact(oversized);
            buf.push_str(&"x".repeat(oversized));
        });

        let mut guard = acquire_string();
        let buf = guard.as_mut();

        assert_string_dropped(buf, MAX_STRING_BUFFER_CAPACITY);
    }
}

mod acquire_bytes {
    use super::*;

    #[test]
    fn should_reuse_byte_buffer_when_capacity_is_preserved_then_retain_previous_capacity() {
        let recorded_capacity = record_byte_capacity(|buf| buf.extend_from_slice(&[1, 2, 3, 4]));

        let mut guard = acquire_bytes();
        let buf = guard.as_mut();

        assert_byte_reuse(buf, recorded_capacity);
    }

    #[test]
    fn should_drop_byte_buffer_when_capacity_exceeds_limit_then_release_oversized_buffer() {
        let oversized = MAX_BYTE_BUFFER_CAPACITY + 4096;
        record_byte_capacity(|buf| {
            buf.reserve_exact(oversized);
            buf.extend(std::iter::repeat_n(0xAB, oversized));
        });

        let mut guard = acquire_bytes();
        let buf = guard.as_mut();

        assert_byte_dropped(buf, MAX_BYTE_BUFFER_CAPACITY);
    }
}
