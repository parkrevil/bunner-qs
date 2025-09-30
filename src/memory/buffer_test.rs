use super::{MAX_BYTE_BUFFER_CAPACITY, MAX_STRING_BUFFER_CAPACITY, acquire_bytes, acquire_string};

#[test]
fn string_guard_reuses_buffer_and_clears_contents() {
    let recorded_capacity;
    {
        let mut guard = acquire_string();
        let buf = guard.as_mut();
        buf.push_str("hello world");
        recorded_capacity = buf.capacity();
    }

    {
        let mut guard = acquire_string();
        let buf = guard.as_mut();
        assert_eq!(buf.len(), 0, "reused buffer should be cleared");
        assert!(
            buf.capacity() >= recorded_capacity,
            "buffer should retain at least prior capacity"
        );
    }
}

#[test]
fn string_guard_discards_oversized_buffer() {
    let oversized_capacity = MAX_STRING_BUFFER_CAPACITY + 1024;

    {
        let mut guard = acquire_string();
        let buf = guard.as_mut();
        buf.reserve(oversized_capacity);
        assert!(buf.capacity() > MAX_STRING_BUFFER_CAPACITY);
    }

    {
        let mut guard = acquire_string();
        let buf = guard.as_mut();
        assert!(
            buf.capacity() <= MAX_STRING_BUFFER_CAPACITY,
            "oversized buffers should not be reused"
        );
        assert_eq!(buf.len(), 0);
    }
}

#[test]
fn byte_guard_reuses_buffer_and_clears_contents() {
    let recorded_capacity;
    {
        let mut guard = acquire_bytes();
        let buf = guard.as_mut();
        buf.extend_from_slice(&[1, 2, 3, 4]);
        recorded_capacity = buf.capacity();
    }

    {
        let mut guard = acquire_bytes();
        let buf = guard.as_mut();
        assert_eq!(buf.len(), 0, "byte buffer should be cleared on reuse");
        assert!(
            buf.capacity() >= recorded_capacity,
            "byte buffer should retain previous capacity"
        );
    }
}

#[test]
fn byte_guard_discards_oversized_buffer() {
    let oversized_capacity = MAX_BYTE_BUFFER_CAPACITY + 4096;

    {
        let mut guard = acquire_bytes();
        let buf = guard.as_mut();
        buf.reserve(oversized_capacity);
        assert!(buf.capacity() > MAX_BYTE_BUFFER_CAPACITY);
    }

    {
        let mut guard = acquire_bytes();
        let buf = guard.as_mut();
        assert!(
            buf.capacity() <= MAX_BYTE_BUFFER_CAPACITY,
            "oversized byte buffers should not be reused"
        );
        assert_eq!(buf.len(), 0);
    }
}
