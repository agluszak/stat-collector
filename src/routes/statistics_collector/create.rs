use axum::{extract::State, http::StatusCode, response::Json};
use diesel::prelude::*;

use uuid::Uuid;

use crate::routes::util::internal_error;
use crate::{db, json, schema};

/// Creates a new statistics collector
#[utoipa::path(
    post,
    path = "/statistics_collector",
    request_body = json::StatisticsCollector,
    responses(
        (status = 200, description = "Ok"),
    )
)]
pub async fn create_statistics_collector(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(statistics_collector): Json<json::StatisticsCollector>,
) -> Result<(), (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    conn.interact(move |conn| {
        conn.transaction(|conn| {
            let db_statistics_collector = db::StatisticsCollector {
                id: None,
                name: statistics_collector.name.clone(),
            };
            let collector_id = diesel::insert_into(schema::statistics_collectors::table)
                .values(&db_statistics_collector)
                .returning(schema::statistics_collectors::id)
                .execute(conn)?;

            let db_periods = statistics_collector
                .periods
                .iter()
                .map(|period| db::Period {
                    id: None,
                    name: period.name.clone(),
                    start: period.start_date,
                    end: period.end_date,
                    statistics_collector_id: collector_id as i32,
                })
                .collect::<Vec<db::Period>>();

            diesel::insert_into(schema::periods::table)
                .values(&db_periods)
                .execute(conn)?;

            let db_placement_types = statistics_collector
                .placement_types
                .iter()
                .map(|placement_type| db::PlacementType {
                    id: None,
                    name: placement_type.name.clone(),
                    statistics_collector_id: collector_id as i32,
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
                                .id
                                .unwrap();
                            let input_page = Uuid::now_v7();
                            db::Supplier {
                                id: None,
                                name: supplier.name.clone(),
                                mail: supplier.mail.clone(),
                                input_page,
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
                                .id
                                .unwrap();
                            db::StatisticType {
                                id: None,
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
                                .id
                                .unwrap();
                            db::Copy {
                                id: None,
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
    .await
    .map_err(internal_error)?
    .map_err(internal_error::<diesel::result::Error>)?;
    Ok(())
}
