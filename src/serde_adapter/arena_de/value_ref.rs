use crate::parsing::arena::ArenaValue;

#[derive(Clone, Copy)]
pub(crate) enum ArenaValueRef<'de> {
    String(&'de str),
    Seq(&'de [ArenaValue<'de>]),
    Map(&'de [(&'de str, ArenaValue<'de>)]),
}

impl<'de> ArenaValueRef<'de> {
    pub(crate) fn from_value(value: &'de ArenaValue<'de>) -> Self {
        match value {
            ArenaValue::String(s) => ArenaValueRef::String(s),
            ArenaValue::Seq(_) => ArenaValueRef::Seq(value.as_seq_slice().unwrap()),
            ArenaValue::Map { .. } => ArenaValueRef::Map(value.as_map_slice().unwrap()),
        }
    }
}
