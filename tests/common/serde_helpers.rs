use bunner_qs::{ParseError, SerdeStringifyError, parse, stringify};
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn assert_encoded_contains(encoded: &str, expected: &[&str]) {
    for fragment in expected {
        assert!(
            encoded.contains(fragment),
            "encoded string `{encoded}` should contain `{fragment}`"
        );
    }
}

pub fn roundtrip_via_public_api<T>(value: &T) -> Result<T, RoundtripError>
where
    T: Serialize + DeserializeOwned + Default,
{
    let encoded = stringify(value).map_err(RoundtripError::from_stringify)?;
    let parsed = parse(&encoded).map_err(RoundtripError::from_parse)?;
    Ok(parsed)
}

#[derive(Debug)]
pub enum RoundtripError {
    Stringify(SerdeStringifyError),
    Parse(ParseError),
}

impl RoundtripError {
    fn from_parse(err: ParseError) -> Self {
        Self::Parse(err)
    }

    fn from_stringify(err: SerdeStringifyError) -> Self {
        Self::Stringify(err)
    }
}

impl std::fmt::Display for RoundtripError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoundtripError::Stringify(err) => write!(f, "stringify error: {err}"),
            RoundtripError::Parse(err) => write!(f, "parse error: {err}"),
        }
    }
}

impl std::error::Error for RoundtripError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RoundtripError::Stringify(err) => Some(err),
            RoundtripError::Parse(err) => Some(err),
        }
    }
}
