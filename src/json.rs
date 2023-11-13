use serde::Deserialize;
use serde::Serialize;
use time::Date;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticsCollector {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Periods")]
    pub periods: Vec<Period>,
    #[serde(rename = "placementTypes")]
    pub placement_types: Vec<PlacementType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Period {
    pub name: String,
    #[serde(rename = "startDate")]
    #[serde(with = "date_serde")]
    pub start_date: Date,
    #[serde(rename = "endDate")]
    #[serde(with = "date_serde")]
    pub end_date: Date,
}

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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacementType {
    pub name: String,
    pub suppliers: Vec<Supplier>,
    pub statistics: Vec<String>,
    pub copies: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Supplier {
    pub name: String,
    pub mail: String,
}
