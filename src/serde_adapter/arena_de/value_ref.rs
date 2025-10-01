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
            ArenaValue::Seq(items) => ArenaValueRef::Seq(items.as_slice()),
            ArenaValue::Map { entries, .. } => ArenaValueRef::Map(entries.as_slice()),
        }
    }
}

#[cfg(test)]
#[path = "value_ref_test.rs"]
mod value_ref_test;
