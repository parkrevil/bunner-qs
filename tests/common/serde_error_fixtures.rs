use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct BoolField {
    pub secure: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct NestedWrapper {
    pub peer: NestedPeer,
}

#[derive(Debug, Deserialize, Default)]
pub struct NestedPeer {
    pub host: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UnitHolder {
    pub empty: (),
}
