use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use lettre::Address;
use time::Date;
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsCollector {
    pub name: String,
    pub client: String,
    pub periods: Vec<Period>,
    pub placement_types: Vec<PlacementType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub name: String,
    #[serde(with = "date_serde")]
    #[schema(example = "2021.01.01")]
    pub start_date: Date,
    #[serde(with = "date_serde")]
    #[schema(example = "2021.12.25")]
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

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PlacementType {
    pub name: String,
    pub suppliers: Vec<Supplier>,
    pub statistics: Vec<String>,
    pub copies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Supplier {
    pub name: String,
    pub mail: Address,
}

/// Stats from a supplier
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct CollectedStats {
    pub periods: Vec<Period>,
    pub copies: Vec<String>,
    /// Outer index is copy, inner index is date
    /// In other words, given dates 1, 2 and copies A, B:
    /// stats[0][0] is the number of copies of A on date 1
    /// stats[0][1] is the number of copies of A on date 2
    /// stats[1][0] is the number of copies of B on date 1
    /// stats[1][1] is the number of copies of B on date 2
    pub stats: BTreeMap<String, Vec<Vec<i32>>>,
}
