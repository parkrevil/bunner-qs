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
    if !buf.is_empty() {
        panic!("reused buffer should be cleared");
    }
    if buf.capacity() < min_capacity {
        panic!("buffer should retain at least prior capacity");
    }
}

fn assert_string_dropped(buf: &String, max_capacity: usize) {
    if buf.capacity() > max_capacity {
        panic!("oversized buffers should not be reused");
    }
    if !buf.is_empty() {
        panic!("dropped buffers should be cleared");
    }
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
    if !buf.is_empty() {
        panic!("byte buffer should be cleared on reuse");
    }
    if buf.capacity() < min_capacity {
        panic!("byte buffer should retain previous capacity");
    }
}

fn assert_byte_dropped(buf: &Vec<u8>, max_capacity: usize) {
    if buf.capacity() > max_capacity {
        panic!("oversized byte buffers should not be reused");
    }
    if !buf.is_empty() {
        panic!("byte buffers should be cleared when dropped");
    }
}

mod acquire_string {
    use super::*;

    #[test]
    fn given_buffer_with_previous_capacity_when_acquire_string_then_reuses_capacity() {
        let recorded_capacity = record_string_capacity(|buf| buf.push_str("hello world"));

        let mut guard = acquire_string();
        let buf = guard.as_mut();

        assert_string_reuse(buf, recorded_capacity);
    }

    #[test]
    fn given_oversized_capacity_when_acquire_string_then_drops_buffer() {
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
    fn given_buffer_with_previous_capacity_when_acquire_bytes_then_reuses_capacity() {
        let recorded_capacity = record_byte_capacity(|buf| buf.extend_from_slice(&[1, 2, 3, 4]));

        let mut guard = acquire_bytes();
        let buf = guard.as_mut();

        assert_byte_reuse(buf, recorded_capacity);
    }

    #[test]
    fn given_oversized_capacity_when_acquire_bytes_then_drops_buffer() {
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

mod assert_string_reuse {
    use super::*;

    #[test]
    #[should_panic(expected = "reused buffer should be cleared")]
    fn given_uncleared_buffer_when_assert_string_reuse_then_panics() {
        let mut guard = acquire_string();
        let buf = guard.as_mut();
        buf.push_str("dirty");

        assert_string_reuse(buf, 0);
    }

    #[test]
    #[should_panic(expected = "buffer should retain at least prior capacity")]
    fn given_smaller_capacity_requirement_when_assert_string_reuse_then_panics() {
        let mut guard = acquire_string();
        let buf = guard.as_mut();
        let minimum = buf.capacity() + 1;

        assert_string_reuse(buf, minimum);
    }
}

mod assert_string_dropped {
    use super::*;

    #[test]
    #[should_panic(expected = "oversized buffers should not be reused")]
    fn given_oversized_buffer_when_assert_string_dropped_then_panics() {
        let mut guard = acquire_string();
        {
            let buf = guard.as_mut();
            buf.reserve_exact(MAX_STRING_BUFFER_CAPACITY + 1024);
        }
        let buf = guard.as_mut();

        assert_string_dropped(buf, MAX_STRING_BUFFER_CAPACITY);
    }

    #[test]
    #[should_panic(expected = "dropped buffers should be cleared")]
    fn given_uncleared_buffer_when_assert_string_dropped_then_panics() {
        let mut guard = acquire_string();
        let buf = guard.as_mut();
        buf.push_str("dirty");
        let maximum = buf.capacity();

        assert_string_dropped(buf, maximum);
    }
}

mod assert_byte_reuse {
    use super::*;

    #[test]
    #[should_panic(expected = "byte buffer should be cleared on reuse")]
    fn given_uncleared_buffer_when_assert_byte_reuse_then_panics() {
        let mut guard = acquire_bytes();
        let buf = guard.as_mut();
        buf.extend_from_slice(&[1, 2, 3]);

        assert_byte_reuse(buf, 0);
    }

    #[test]
    #[should_panic(expected = "byte buffer should retain previous capacity")]
    fn given_smaller_capacity_requirement_when_assert_byte_reuse_then_panics() {
        let mut guard = acquire_bytes();
        let buf = guard.as_mut();
        let minimum = buf.capacity() + 1;

        assert_byte_reuse(buf, minimum);
    }
}

mod assert_byte_dropped {
    use super::*;

    #[test]
    #[should_panic(expected = "oversized byte buffers should not be reused")]
    fn given_oversized_buffer_when_assert_byte_dropped_then_panics() {
        let mut guard = acquire_bytes();
        {
            let buf = guard.as_mut();
            buf.reserve_exact(MAX_BYTE_BUFFER_CAPACITY + 4096);
        }
        let buf = guard.as_mut();

        assert_byte_dropped(buf, MAX_BYTE_BUFFER_CAPACITY);
    }

    #[test]
    #[should_panic(expected = "byte buffers should be cleared when dropped")]
    fn given_uncleared_buffer_when_assert_byte_dropped_then_panics() {
        let mut guard = acquire_bytes();
        let buf = guard.as_mut();
        buf.extend_from_slice(&[1, 2, 3]);
        let maximum = buf.capacity();

        assert_byte_dropped(buf, maximum);
    }
}
