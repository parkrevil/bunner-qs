use std::cell::RefCell;

thread_local! {
    static STRING_BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
    static BYTE_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

pub(crate) struct StringGuard {
    buffer: Option<String>,
}

impl StringGuard {
    pub(crate) fn as_mut(&mut self) -> &mut String {
        self.buffer.as_mut().unwrap()
    }
}

impl Drop for StringGuard {
    fn drop(&mut self) {
        if let Some(mut buf) = self.buffer.take() {
            buf.clear();
            STRING_BUFFER.with(|cell| {
                let mut storage = cell.borrow_mut();
                *storage = buf;
            });
        }
    }
}

pub(crate) struct ByteGuard {
    buffer: Option<Vec<u8>>,
}

impl ByteGuard {
    pub(crate) fn as_mut(&mut self) -> &mut Vec<u8> {
        self.buffer.as_mut().unwrap()
    }
}

impl Drop for ByteGuard {
    fn drop(&mut self) {
        if let Some(mut buf) = self.buffer.take() {
            buf.clear();
            BYTE_BUFFER.with(|cell| {
                let mut storage = cell.borrow_mut();
                *storage = buf;
            });
        }
    }
}

pub(crate) fn acquire_string() -> StringGuard {
    STRING_BUFFER.with(|cell| {
        let mut storage = cell.borrow_mut();
        let buf = std::mem::take(&mut *storage);
        drop(storage);
        StringGuard { buffer: Some(buf) }
    })
}

pub(crate) fn acquire_bytes() -> ByteGuard {
    BYTE_BUFFER.with(|cell| {
        let mut storage = cell.borrow_mut();
        let buf = std::mem::take(&mut *storage);
        drop(storage);
        ByteGuard { buffer: Some(buf) }
    })
}
