use std::sync::{Arc, Mutex};
use axum::{extract::State, response::Json};
use diesel::prelude::*;
use time::OffsetDateTime;

use crate::db::{CopyId, PeriodId, PlacementTypeId, StatCollectorId, StatisticTypeId, SupplierId};

use crate::errors::AppError;
use crate::{db, json, schema};
use crate::logic::email::Mailer;
use crate::logic::time::Clock;

/// Creates a new statistics collector
#[utoipa::path(
    post,
    path = "/statistics_collector",
    request_body = StatisticsCollector,
    responses(
        (status = 200, description = "Ok"),
    )
)]
pub async fn create_statistics_collector(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    State(clock): State<Arc<Mutex<dyn Clock>>>,
    Json(statistics_collector): Json<json::received::StatCollector>,
) -> Result<Json<StatCollectorId>, AppError> {
    let conn = pool.get().await?;
    let id = conn
        .interact(move |conn| {
            conn.transaction(|conn| {
                let collector_id = StatCollectorId::new();

                let db_statistics_collector = db::StatisticsCollector {
                    id: collector_id,
                    periodicity: statistics_collector.periodicity.clone(),
                    weekday: statistics_collector.weekday.clone(),
                    name: statistics_collector.name.clone(),
                    client: statistics_collector.client.clone(),
                };

                // Ensure that (name, client) tuple is unique
                let existing = schema::statistics_collectors::table
                    .select(schema::statistics_collectors::id)
                    .filter(
                        schema::statistics_collectors::name
                            .eq(&db_statistics_collector.name)
                            .and(
                                schema::statistics_collectors::client
                                    .eq(&db_statistics_collector.client),
                            ),
                    )
                    .first::<StatCollectorId>(conn)
                    .optional()?;

                if let Some(existing) = existing {
                    return Err(AppError::Conflict {
                        resource: format!(
                            "statistics collector with name {} and client {}",
                            db_statistics_collector.name, db_statistics_collector.client
                        ),
                        id: existing.to_string(),
                    });
                }

                diesel::insert_into(schema::statistics_collectors::table)
                    .values(&db_statistics_collector)
                    .execute(conn)?;

                let db_periods = statistics_collector
                    .periods
                    .iter()
                    .map(|period| db::Period {
                        id: PeriodId::new(),
                        name: period.name.clone(),
                        start: period.start_date,
                        end: period.end_date,
                        statistics_collector_id: collector_id,
                    })
                    .collect::<Vec<db::Period>>();

                diesel::insert_into(schema::periods::table)
                    .values(&db_periods)
                    .execute(conn)?;

                let db_placement_types = statistics_collector
                    .placement_types
                    .iter()
                    .map(|placement_type| db::PlacementType {
                        id: PlacementTypeId::new(),
                        name: placement_type.name.clone(),
                        statistics_collector_id: collector_id,
                    })
                    .collect::<Vec<db::PlacementType>>();

                let db_placement_types = diesel::insert_into(schema::placement_types::table)
                    .values(&db_placement_types)
                    .get_results::<db::PlacementType>(conn)?;

                let db_suppliers = statistics_collector
                    .placement_types
                    .iter()
                    .flat_map(|placement_type| {
                        placement_type
                            .suppliers
                            .iter()
                            .map(|supplier| {
                                let placement_type_id = db_placement_types
                                    .iter()
                                    .find(|db_placement_type| {
                                        db_placement_type.name == placement_type.name
                                    })
                                    .unwrap()
                                    .id;
                                db::Supplier {
                                    id: SupplierId::new(),
                                    name: supplier.name.clone(),
                                    mail: supplier.mail.to_string(),
                                    placement_type_id,
                                    submitted_date: clock.lock().unwrap().now(),
                                }
                            })
                            .collect::<Vec<db::Supplier>>()
                    })
                    .collect::<Vec<db::Supplier>>();

                let _db_suppliers = diesel::insert_into(schema::suppliers::table)
                    .values(&db_suppliers)
                    .get_results::<db::Supplier>(conn)?;

                let db_statistic_types = statistics_collector
                    .placement_types
                    .iter()
                    .flat_map(|placement_type| {
                        placement_type
                            .statistics
                            .iter()
                            .map(|statistic| {
                                let placement_type_id = db_placement_types
                                    .iter()
                                    .find(|db_placement_type| {
                                        db_placement_type.name == placement_type.name
                                    })
                                    .unwrap()
                                    .id;
                                db::StatisticType {
                                    id: StatisticTypeId::new(),
                                    name: statistic.clone(),
                                    placement_type_id,
                                }
                            })
                            .collect::<Vec<db::StatisticType>>()
                    })
                    .collect::<Vec<db::StatisticType>>();

                diesel::insert_into(schema::statistic_types::table)
                    .values(&db_statistic_types)
                    .execute(conn)?;

                let db_copies = statistics_collector
                    .placement_types
                    .iter()
                    .flat_map(|placement_type| {
                        placement_type
                            .copies
                            .iter()
                            .map(|copy| {
                                let placement_type_id = db_placement_types
                                    .iter()
                                    .find(|db_placement_type| {
                                        db_placement_type.name == placement_type.name
                                    })
                                    .unwrap()
                                    .id;
                                db::Copy {
                                    id: CopyId::new(),
                                    name: copy.clone(),
                                    placement_type_id,
                                }
                            })
                            .collect::<Vec<db::Copy>>()
                    })
                    .collect::<Vec<db::Copy>>();

                diesel::insert_into(schema::copies::table)
                    .values(&db_copies)
                    .execute(conn)?;

                // for each period, for supplier, for each of supplier's statistic types, for each of supplier's copies
                let db_statistics = db_periods
                    .iter()
                    .flat_map(|period| {
                        db_suppliers
                            .iter()
                            .flat_map(|supplier| {
                                db_statistic_types
                                    .iter()
                                    .filter(|statistic_type| {
                                        statistic_type.placement_type_id
                                            == supplier.placement_type_id
                                    })
                                    .flat_map(|statistic_type| {
                                        db_copies
                                            .iter()
                                            .filter(|copy| {
                                                copy.placement_type_id == supplier.placement_type_id
                                            })
                                            .map(|copy| db::Statistic {
                                                value: 0,
                                                period_id: period.id,
                                                supplier_id: supplier.id,
                                                statistic_type_id: statistic_type.id,
                                                copy_id: copy.id,
                                            })
                                            .collect::<Vec<db::Statistic>>()
                                    })
                                    .collect::<Vec<db::Statistic>>()
                            })
                            .collect::<Vec<db::Statistic>>()
                    })
                    .collect::<Vec<db::Statistic>>();

                diesel::insert_into(schema::statistics::table)
                    .values(&db_statistics)
                    .execute(conn)?;

                Ok(collector_id)
            })
        })
        .await??;
    Ok(Json(id))
}
