use axum::{extract::State, response::Json};
use diesel::prelude::*;

use crate::db::{CopyId, PeriodId, PlacementTypeId, StatCollectorId, StatisticTypeId, SupplierId};

use crate::routes::errors::AppError;
use crate::{db, json, schema};

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
    Json(statistics_collector): Json<json::StatisticsCollector>,
) -> Result<(), AppError> {
    let conn = pool.get().await?;
    conn.interact(move |conn| {
        conn.transaction(|conn| {
            let collector_id = StatCollectorId::new();

            let db_statistics_collector = db::StatisticsCollector {
                id: collector_id,
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
                                mail: supplier.mail.clone(),
                                placement_type_id,
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

            Ok(())
        })
    })
    .await??;
    Ok(())
}
