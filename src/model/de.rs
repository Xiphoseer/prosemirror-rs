use serde::de::{Deserialize, Deserializer};

pub fn deserialize_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    let opt_string: Option<T> = Deserialize::deserialize(deserializer)?;
    Ok(opt_string.unwrap_or_default())
}
