use crate::util::is_ascii_control;

pub(crate) fn ensure_no_control(value: &str) -> Result<(), ()> {
    if value.chars().any(is_ascii_control) {
        Err(())
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "validate_test.rs"]
mod validate_test;
