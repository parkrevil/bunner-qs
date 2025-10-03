#[inline]
pub(crate) const fn is_ascii_control(ch: char) -> bool {
    matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}')
}
