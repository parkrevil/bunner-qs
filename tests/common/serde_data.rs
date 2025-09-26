use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub use flattened::{FlattenedContact, FlattenedName, FlattenedProfile};
pub use forms::{ContactForm, NetworkPeer, ProfileForm, SimpleUser, TaggedRecord};
pub use preferred::{DesiredContact, DesiredPhone, DesiredProfile};
pub use tagging::{NotificationPreference, TaggedSettings};
pub use utils::{deserialize_trimmed, serialize_trimmed};

mod forms {
    use super::*;

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
    pub struct ContactForm {
        #[serde(rename = "email📧", alias = "email_address")]
        pub email: String,
        #[serde(rename = "primary-phone")]
        pub primary_phone: String,
        #[serde(
            default,
            alias = "secondaryPhone",
            skip_serializing_if = "Option::is_none"
        )]
        pub secondary_phone: Option<String>,
    }

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
    pub struct ProfileForm {
        #[serde(rename = "user_name")]
        pub username: String,
        #[serde(rename = "age-years", alias = "user_age")]
        pub age: u8,
        #[serde(rename = "active?")]
        pub active: bool,
        #[serde(rename = "contact📞")]
        pub contact: ContactForm,
        #[serde(
            default,
            rename = "nickname🎭",
            alias = "alias🎭",
            skip_serializing_if = "Option::is_none"
        )]
        pub nickname: Option<String>,
    }

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
    pub struct SimpleUser {
        pub username: String,
        pub age: u8,
        pub active: bool,
    }

    #[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
    pub struct TaggedRecord {
        pub name: String,
        pub tags: Vec<String>,
    }

    #[derive(Debug, Deserialize, PartialEq, Default)]
    pub struct NetworkPeer {
        pub host: String,
        pub port: u16,
        pub secure: bool,
    }
}

mod preferred {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    pub struct DesiredPhone {
        #[serde(rename = "kind🥇")]
        pub kind: String,
        #[serde(rename = "number#")]
        pub number: String,
        #[serde(rename = "preferred✔")]
        pub preferred: bool,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    pub struct DesiredContact {
        #[serde(rename = "email📮")]
        pub email: String,
        #[serde(rename = "phones📱")]
        pub phones: Vec<String>,
        #[serde(rename = "numbers📇")]
        pub numbers: Vec<DesiredPhone>,
        #[serde(rename = "tags🔥", alias = "tag_list", default)]
        pub tags: Vec<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    pub struct DesiredProfile {
        #[serde(rename = "profile✨name")]
        pub username: String,
        #[serde(rename = "age✨", alias = "alt_age")]
        pub age: u8,
        pub contact: DesiredContact,
        pub bio: Option<String>,
    }
}

mod flattened {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    pub struct FlattenedName {
        #[serde(rename = "first_name")]
        pub first: String,
        #[serde(rename = "last_name")]
        pub last: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    pub struct FlattenedContact {
        #[serde(rename = "contact_email")]
        pub email: String,
        #[serde(rename = "contact_phone")]
        pub phone: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
    pub struct FlattenedProfile {
        #[serde(flatten)]
        pub name: FlattenedName,
        #[serde(flatten)]
        pub contact: FlattenedContact,
        pub active: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub note: Option<String>,
    }
}

mod tagging {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    #[serde(tag = "notification_kind", content = "notification")]
    pub enum NotificationPreference {
        Email { address: String },
        Sms { number: String },
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    pub struct TaggedSettings {
        #[serde(flatten)]
        pub preference: NotificationPreference,
        #[serde(
            rename = "access_token",
            serialize_with = "super::serialize_trimmed",
            deserialize_with = "super::deserialize_trimmed"
        )]
        pub token: String,
    }

    impl Default for TaggedSettings {
        fn default() -> Self {
            Self {
                preference: NotificationPreference::Email {
                    address: String::new(),
                },
                token: String::new(),
            }
        }
    }
}

mod utils {
    use super::*;

    pub fn serialize_trimmed<S>(value: &str, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value.trim())
    }

    pub fn deserialize_trimmed<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(raw.trim().to_string())
    }
}
