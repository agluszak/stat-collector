#![allow(clippy::new_without_default)]

use crate::json;
use derive_more::Display;
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};

use chrono::{DateTime, Local, NaiveDate};
use utoipa::ToResponse;
use uuid::Uuid;

use crate::schema::*;

#[repr(transparent)]
#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DieselNewType,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Display,
    ToResponse,
)]
pub struct StatCollectorId(Uuid);

impl StatCollectorId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for StatCollectorId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(
    Debug, PartialEq, Serialize, Deserialize, Queryable, Selectable, Identifiable, Insertable, Clone,
)]
#[diesel(table_name = statistics_collectors)]
pub struct StatisticsCollector {
    pub id: StatCollectorId,
    pub name: String,
    pub client: String,
    pub periodicity: String,
    pub weekday: String,
}

#[repr(transparent)]
#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DieselNewType,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Display,
)]
pub struct PeriodId(Uuid);

impl PeriodId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(StatisticsCollector))]
#[diesel(table_name = periods)]
pub struct Period {
    pub id: PeriodId,
    pub name: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub statistics_collector_id: StatCollectorId,
}

impl Period {
    pub fn as_json(&self) -> json::sent::Period {
        json::sent::Period {
            id: self.id,
            name: self.name.clone(),
            start_date: self.start,
            end_date: self.end,
        }
    }
}

#[repr(transparent)]
#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DieselNewType,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Display,
)]
pub struct PlacementTypeId(Uuid);

impl PlacementTypeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(
    Debug,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Queryable,
    Selectable,
    Identifiable,
    Associations,
    Insertable,
)]
#[diesel(table_name = placement_types)]
#[diesel(belongs_to(StatisticsCollector))]
pub struct PlacementType {
    pub id: PlacementTypeId,
    pub name: String,
    pub statistics_collector_id: StatCollectorId,
}

#[repr(transparent)]
#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DieselNewType,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Display,
)]
pub struct SupplierId(Uuid);

impl SupplierId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = suppliers)]
#[diesel(belongs_to(PlacementType))]
pub struct Supplier {
    pub id: SupplierId,
    pub name: String,
    pub mail: String,
    pub placement_type_id: PlacementTypeId,
    pub submitted_date: DateTime<Local>,
}

impl Supplier {
    pub fn as_json(&self, stats: Vec<Vec<Vec<i32>>>) -> json::sent::Supplier {
        json::sent::Supplier {
            id: self.id,
            name: self.name.clone(),
            mail: self.mail.parse().unwrap(),
            stats,
        }
    }
}

#[repr(transparent)]
#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DieselNewType,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Display,
)]
pub struct StatisticTypeId(Uuid);

impl StatisticTypeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = statistic_types)]
#[diesel(belongs_to(PlacementType))]
pub struct StatisticType {
    pub id: StatisticTypeId,
    pub name: String,
    pub placement_type_id: PlacementTypeId,
}

impl StatisticType {
    pub fn as_json(&self) -> String {
        self.name.clone()
    }
}

#[repr(transparent)]
#[derive(
    Debug,
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    DieselNewType,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Display,
)]
pub struct CopyId(Uuid);

impl CopyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = copies)]
#[diesel(belongs_to(PlacementType))]
pub struct Copy {
    pub id: CopyId,
    pub name: String,
    pub placement_type_id: PlacementTypeId,
}

impl Copy {
    pub fn as_json(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = statistics)]
#[diesel(belongs_to(Period))]
#[diesel(belongs_to(Supplier))]
#[diesel(belongs_to(StatisticType))]
#[diesel(belongs_to(Copy))]
#[diesel(primary_key(period_id, supplier_id, statistic_type_id, copy_id))]
pub struct Statistic {
    pub period_id: PeriodId,
    pub supplier_id: SupplierId,
    pub statistic_type_id: StatisticTypeId,
    pub copy_id: CopyId,
    pub value: i32,
}
