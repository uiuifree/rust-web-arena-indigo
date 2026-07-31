//! Serde helpers absorbing the Indigo API's loose typing: some parameters must
//! be sent as JSON strings, and some response fields switch between numbers
//! and strings.

use serde::{Deserialize, Deserializer, Serializer};

/// Serializes a number as a JSON string, as required by several endpoints.
pub(crate) fn u32_as_string<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub(crate) fn u32_vec_as_strings<S>(values: &[u32], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_seq(values.iter().map(|v| v.to_string()))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NumberOrString<T> {
    Number(T),
    String(String),
}

fn lenient<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + std::str::FromStr,
    T::Err: std::fmt::Display,
    D: Deserializer<'de>,
{
    match NumberOrString::<T>::deserialize(deserializer)? {
        NumberOrString::Number(v) => Ok(v),
        NumberOrString::String(s) => s.parse().map_err(serde::de::Error::custom),
    }
}

/// Deserializes a field that the API returns as either a number or a string.
pub(crate) fn u32_lenient<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    lenient(deserializer)
}

pub(crate) fn u64_lenient<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    lenient(deserializer)
}

/// Deserializes an optional field that may arrive as a number or a string,
/// normalizing to a string.
pub(crate) fn opt_string_lenient<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Number(i64),
        String(String),
    }
    Ok(
        Option::<Value>::deserialize(deserializer)?.map(|v| match v {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s,
        }),
    )
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Lenient {
        #[serde(deserialize_with = "super::u32_lenient")]
        small: u32,
        #[serde(deserialize_with = "super::u64_lenient")]
        large: u64,
        #[serde(default, deserialize_with = "super::opt_string_lenient")]
        text: Option<String>,
    }

    #[test]
    fn accepts_numbers_and_strings() {
        let v: Lenient =
            serde_json::from_str(r#"{"small": "12", "large": "1657781876000", "text": 3}"#)
                .unwrap();
        assert_eq!(v.small, 12);
        assert_eq!(v.large, 1657781876000);
        assert_eq!(v.text.as_deref(), Some("3"));

        let v: Lenient =
            serde_json::from_str(r#"{"small": 12, "large": 1657781876000, "text": "3"}"#).unwrap();
        assert_eq!(v.small, 12);
        assert_eq!(v.large, 1657781876000);
        assert_eq!(v.text.as_deref(), Some("3"));

        let v: Lenient = serde_json::from_str(r#"{"small": 0, "large": 0, "text": null}"#).unwrap();
        assert_eq!(v.text, None);
    }

    #[test]
    fn rejects_non_numeric_strings() {
        assert!(serde_json::from_str::<Lenient>(r#"{"small": "x", "large": 0}"#).is_err());
    }
}
