use diesel::prelude::*;
use time::Date;
use crate::schema::*;

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = statistics_collectors)]
pub struct StatisticsCollector {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(belongs_to(StatisticsCollector))]
#[diesel(table_name = periods)]
pub struct Period {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub start: Date,
    pub end: Date,
    pub statistics_collector_id: i32,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = placement_types)]
#[diesel(belongs_to(StatisticsCollector))]
pub struct PlacementType {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub statistics_collector_id: i32,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = suppliers)]
#[diesel(belongs_to(PlacementType))]
pub struct Supplier {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub mail: String,
    pub placement_type_id: i32,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = statistic_types)]
#[diesel(belongs_to(PlacementType))]
pub struct StatisticType {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub placement_type_id: i32,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = copies)]
#[diesel(belongs_to(PlacementType))]
pub struct Copy {
    #[diesel(deserialize_as = i32)]
    pub id: Option<i32>,
    pub name: String,
    pub placement_type_id: i32,
}

#[derive(Debug, PartialEq, Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = statistics)]
#[diesel(belongs_to(Period))]
#[diesel(belongs_to(Supplier))]
#[diesel(belongs_to(StatisticType))]
#[diesel(belongs_to(Copy))]
#[diesel(primary_key(period_id, supplier_id, statistic_type_id, copy_id))]
pub struct Statistic {
    pub period_id: i32,
    pub supplier_id: i32,
    pub statistic_type_id: i32,
    pub copy_id: i32,
    pub value: i32,
}
