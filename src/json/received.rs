use crate::json::date_serde;
use lettre::Address;
use serde::Deserialize;
use serde::Serialize;

use chrono::NaiveDate;
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StatCollector {
    pub name: String,
    pub client: String,
    pub periods: Vec<Period>,
    pub placement_types: Vec<PlacementType>,
    pub periodicity: String,
    pub weekday: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    pub name: String,
    #[serde(with = "date_serde")]
    #[schema(example = "2021.01.01")]
    pub start_date: NaiveDate,
    #[serde(with = "date_serde")]
    #[schema(example = "2021.12.25")]
    pub end_date: NaiveDate,
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
