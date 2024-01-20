pub mod received;
pub mod sent;

mod date_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use time::format_description::FormatItem;
    use time::macros::format_description;
    use time::Date;

    const FORMAT: &[FormatItem] = format_description!("[year].[month].[day]");

    pub fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Date::parse(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
