use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct BoolField {
    pub secure: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct NestedWrapper {
    pub peer: NestedPeer,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct NestedPeer {
    pub host: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
pub struct UnitHolder {
    pub empty: (),
}
