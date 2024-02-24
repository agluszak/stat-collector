use crate::db::{PeriodId, PlacementTypeId, StatCollectorId, SupplierId};
use crate::json::date_serde;
use chrono::NaiveDate;
use lettre::Address;
use serde::Deserialize;
use serde::Serialize;
use utoipa::{ToResponse, ToSchema};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema, ToResponse)]
#[serde(rename_all = "camelCase")]
pub struct StatCollector {
    pub id: StatCollectorId,
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
    pub id: PeriodId,
    pub name: String,
    #[serde(with = "date_serde")]
    #[schema(example = "2021.01.01")]
    pub start_date: NaiveDate,
    #[serde(with = "date_serde")]
    #[schema(example = "2021.12.25")]
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PlacementType {
    pub id: PlacementTypeId,
    pub name: String,
    pub suppliers: Vec<Supplier>,
    pub statistics: Vec<String>,
    pub copies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Supplier {
    pub id: SupplierId,
    pub name: String,
    pub mail: Address,
    /// Outer index is stat type, middle index is copy inner index is date
    /// In other words, given stat types Display and Clicks, dates 1, 2, 3 and copies A, B:
    /// stats[0][0][0] is the number of displays for copy A on date 1
    /// stats[0][0][1] is the number of displays for copy A on date 2
    /// stats[0][0][2] is the number of displays for copy A on date 3
    /// stats[0][1][0] is the number of displays for copy B on date 1
    /// stats[0][1][1] is the number of displays for copy B on date 2
    /// stats[0][1][2] is the number of displays for copy B on date 3
    /// stats[1][0][0] is the number of clicks for copy A on date 1
    /// stats[1][0][1] is the number of clicks for copy A on date 2
    /// stats[1][0][2] is the number of clicks for copy A on date 3
    /// stats[1][1][0] is the number of clicks for copy B on date 1
    /// stats[1][1][1] is the number of clicks for copy B on date 2
    /// etc.
    pub stats: Vec<Vec<Vec<i32>>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use once_cell::sync::Lazy;

    static PERIOD: Lazy<Period> = Lazy::new(|| Period {
        id: PeriodId::new(),
        name: "test period".to_string(),
        start_date: NaiveDate::from_ymd_opt(2021, 4, 2).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2021, 4, 3).unwrap(),
    });

    static SUPPLIER: Lazy<Supplier> = Lazy::new(|| Supplier {
        id: SupplierId::new(),
        name: "test supplier".to_string(),
        mail: Address::new("user", "test.com").unwrap(),
        stats: vec![vec![vec![0, 1, 2], vec![3, 4, 5]]],
    });

    static PLACEMENT_TYPE: Lazy<PlacementType> = Lazy::new(|| PlacementType {
        id: PlacementTypeId::new(),
        name: "test placement type".to_string(),
        suppliers: vec![SUPPLIER.clone()],
        statistics: vec!["test statistic".to_string()],
        copies: vec!["test copy".to_string()],
    });

    static STAT_COLLECTOR: Lazy<StatCollector> = Lazy::new(|| StatCollector {
        id: StatCollectorId::new(),
        name: "test stat collector".to_string(),
        client: "test client".to_string(),
        periods: vec![PERIOD.clone()],
        placement_types: vec![PLACEMENT_TYPE.clone()],
        periodicity: "idk".to_string(),
        weekday: "saturday".to_string(),
    });

    #[test]
    fn received_stat_collector_can_be_constructed_from_sent() {
        // serialize to json and deserialize as received
        let received: json::received::StatCollector =
            serde_json::from_str(&serde_json::to_string(&*STAT_COLLECTOR).unwrap()).unwrap();
        assert_eq!(received.name, STAT_COLLECTOR.name);
        assert_eq!(received.client, STAT_COLLECTOR.client);
        assert_eq!(received.periods.len(), STAT_COLLECTOR.periods.len());
        assert_eq!(
            received.placement_types.len(),
            STAT_COLLECTOR.placement_types.len()
        );
    }
}
