use crate::json;
use diesel::prelude::*;
use diesel_derive_newtype::DieselNewType;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use time::Date;
use uuid::Uuid;

use crate::schema::*;

#[repr(transparent)]
#[derive(
    Debug, Hash, PartialEq, Eq, PartialOrd, Ord, DieselNewType, Serialize, Deserialize, Clone, Copy,
)]
pub struct StatCollectorId(Uuid);

impl Default for StatCollectorId {
    fn default() -> Self {
        Self::new()
    }
}

impl StatCollectorId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for StatCollectorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, PartialEq, Serialize, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = statistics_collectors)]
pub struct StatisticsCollector {
    pub id: StatCollectorId,
    pub name: String,
}

#[repr(transparent)]
#[derive(
    Debug, Hash, PartialEq, Eq, PartialOrd, Ord, DieselNewType, Serialize, Deserialize, Clone, Copy,
)]
pub struct PeriodId(Uuid);

impl Default for PeriodId {
    fn default() -> Self {
        Self::new()
    }
}

impl PeriodId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Display for PeriodId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(StatisticsCollector))]
#[diesel(table_name = periods)]
pub struct Period {
    pub id: PeriodId,
    pub name: String,
    pub start: Date,
    pub end: Date,
    pub statistics_collector_id: StatCollectorId,
}

impl Period {
    pub fn as_json(&self) -> json::Period {
        json::Period {
            name: self.name.clone(),
            start_date: self.start,
            end_date: self.end,
        }
    }
}

#[repr(transparent)]
#[derive(
    Debug, Hash, PartialEq, Eq, PartialOrd, Ord, DieselNewType, Serialize, Deserialize, Clone, Copy,
)]
pub struct PlacementTypeId(Uuid);

impl Default for PlacementTypeId {
    fn default() -> Self {
        Self::new()
    }
}

impl PlacementTypeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for PlacementTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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
    Debug, Hash, PartialEq, Eq, PartialOrd, Ord, DieselNewType, Serialize, Deserialize, Clone, Copy,
)]
pub struct SupplierId(Uuid);

impl Default for SupplierId {
    fn default() -> Self {
        Self::new()
    }
}

impl SupplierId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Display for SupplierId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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
}

#[repr(transparent)]
#[derive(
    Debug, Hash, PartialEq, Eq, PartialOrd, Ord, DieselNewType, Serialize, Deserialize, Clone, Copy,
)]
pub struct StatisticTypeId(Uuid);

impl Default for StatisticTypeId {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticTypeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Display for StatisticTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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

#[repr(transparent)]
#[derive(
    Debug, Hash, PartialEq, Eq, PartialOrd, Ord, DieselNewType, Serialize, Deserialize, Clone, Copy,
)]
pub struct CopyId(Uuid);

impl Default for CopyId {
    fn default() -> Self {
        Self::new()
    }
}

impl CopyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Display for CopyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
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
