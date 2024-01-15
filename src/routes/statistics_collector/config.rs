
use axum::extract::{Path, State};
use axum::Json;
use crate::db::{StatCollectorId};
use crate::{db, json, schema};
use crate::routes::errors::AppError;

use diesel::prelude::*;
use itertools::Itertools;



/// Returns the same json as the one used to create the statistics collector
#[utoipa::path(
    get,
    path = "/statistics_collector/{collector_id}/config",
    params(
        ("collector_id" = Uuid, Path, description = "Statistics collector id")
    ),
    responses(
        (status = 200, description = "Ok", content_type = "application/json"),
        (status = 404, description = "No such id", content_type = "text/html")
    )
)]
pub async fn get_collector_config(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(collector_id): Path<StatCollectorId>,
) -> Result<Json<json::StatisticsCollector>, AppError> {
    let conn = pool.get().await?;
    let map = conn
        .interact(move |conn| {
            let collector = schema::statistics_collectors::table
                .find(collector_id)
                .first::<db::StatisticsCollector>(conn)
                .map_err(|_| AppError::not_found("collector", collector_id))?;

            let periods = schema::periods::table
                .filter(schema::periods::statistics_collector_id.eq(collector_id))
                .load::<db::Period>(conn)?
                .into_iter()
                .map(|period| period.as_json())
                .collect_vec();

            let placement_types = schema::placement_types::table
                .filter(schema::placement_types::statistics_collector_id.eq(collector_id))
                .load::<db::PlacementType>(conn)?
            ;

            let mut json_placement_types = Vec::new();

            for placement_type in placement_types {
                    let suppliers = schema::suppliers::table
                        .filter(schema::suppliers::placement_type_id.eq(placement_type.id))
                        .load::<db::Supplier>(conn)?
                        .into_iter()
                        .map(|supplier| supplier.as_json())
                        .collect_vec();

                    let copies = schema::copies::table
                        .filter(schema::copies::placement_type_id.eq(placement_type.id))
                        .load::<db::Copy>(conn)?
                        .into_iter()
                        .map(|copy| copy.as_json())
                        .collect_vec();

                    let statistics = schema::statistic_types::table
                        .filter(schema::statistic_types::placement_type_id.eq(placement_type.id))
                        .load::<db::StatisticType>(conn)?
                        .into_iter()
                        .map(|statistic_type| statistic_type.name.clone())
                        .collect_vec();

                    let placement_type = json::PlacementType {
                        name: placement_type.name.clone(),
                        suppliers,
                        copies,
                        statistics,
                    };

                    json_placement_types.push(placement_type);
                }

            Ok::<_, AppError>(json::StatisticsCollector {
                name: collector.name,
                client: collector.client,
                periods,
                placement_types: json_placement_types,
            })
        })
        .await??;

    Ok(Json(map))
}