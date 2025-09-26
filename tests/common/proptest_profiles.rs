use proptest::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RandomContact {
    pub email: String,
    #[serde(default)]
    pub phones: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RandomProfileData {
    pub username: String,
    pub age: u8,
    pub active: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub contact: Option<RandomContact>,
}

fn small_string() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::char::range('a', 'z'), 0..8)
        .prop_map(|chars| chars.into_iter().collect())
}

fn contact_strategy() -> impl Strategy<Value = RandomContact> {
    (small_string(), prop::collection::vec(small_string(), 0..3))
        .prop_map(|(email, phones)| RandomContact { email, phones })
}

pub fn random_profile_strategy() -> impl Strategy<Value = RandomProfileData> {
    (
        small_string(),
        any::<u8>(),
        any::<bool>(),
        prop::collection::vec(small_string(), 0..4),
        prop::option::of(contact_strategy()),
    )
        .prop_map(|(username, age, active, tags, contact)| RandomProfileData {
            username,
            age,
            active,
            tags,
            contact,
        })
}
