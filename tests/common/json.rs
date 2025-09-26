use serde_json::{Map as JsonMap, Value};

pub fn json_from_pairs(pairs: &[(&str, &str)]) -> Value {
    let mut map = JsonMap::new();
    for (key, value) in pairs {
        map.insert((*key).to_owned(), Value::String((*value).to_owned()));
    }
    Value::Object(map)
}
