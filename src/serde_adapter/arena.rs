use super::{SerdeQueryError, deserialize_from_arena_map};
use crate::parsing::arena::{ArenaQueryMap, ArenaValue};
use serde::de::DeserializeOwned;
use serde_json::{Map as JsonMap, Value as JsonValue};

pub fn from_arena_query_map<T: DeserializeOwned>(
    query_map: &ArenaQueryMap<'_>,
) -> Result<T, SerdeQueryError> {
    deserialize_from_arena_map(query_map).map_err(SerdeQueryError::from)
}

pub fn arena_map_to_json_value(query_map: &ArenaQueryMap<'_>) -> JsonValue {
    let mut object = JsonMap::with_capacity(query_map.len());
    for (key, value) in query_map.iter() {
        object.insert(key.to_string(), arena_value_to_json(value));
    }
    JsonValue::Object(object)
}

fn arena_value_to_json(value: &ArenaValue<'_>) -> JsonValue {
    match value {
        ArenaValue::String(s) => JsonValue::String((*s).to_string()),
        ArenaValue::Seq(items) => {
            let array = items.iter().map(arena_value_to_json).collect();
            JsonValue::Array(array)
        }
        ArenaValue::Map { entries, .. } => {
            let mut object = JsonMap::with_capacity(entries.len());
            for (key, value) in entries.iter() {
                object.insert((*key).to_string(), arena_value_to_json(value));
            }
            JsonValue::Object(object)
        }
    }
}
