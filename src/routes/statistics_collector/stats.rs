use crate::routes::util::{internal_error};
use crate::{db, json, schema};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;
use itertools::Itertools;
use std::collections::BTreeMap;

/// Gets statistics for a supplier
#[utoipa::path(
    get,
    path = "/statistics_collector/{supplier_id}/stats",
    params(
        ("supplier_id" = i32, Path, description = "Supplier id")
    ),
    responses(
        (status = 200, description = "Ok", body = json::CollectedStats, content_type = "application/json"),
        (status = 404, description = "No such id", content_type = "text/html")
    )
)]
pub async fn get_stats(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(supplier_id): Path<i32>,
) -> Result<Json<json::CollectedStats>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let stats = conn
        .interact(move |conn| {
            conn.transaction(|conn| {
                let placement_type_id = schema::suppliers::table
                    .filter(schema::suppliers::id.eq(supplier_id))
                    .select(schema::suppliers::placement_type_id)
                    .first::<i32>(conn)?; // TODO: not found

                let collector_id = schema::placement_types::table
                    .filter(schema::placement_types::id.eq(placement_type_id))
                    .select(schema::placement_types::statistics_collector_id)
                    .first::<i32>(conn)?;

                let copies = schema::copies::table
                    .filter(schema::copies::placement_type_id.eq(placement_type_id))
                    .load::<db::Copy>(conn)?
                    .into_iter()
                    .sorted_by_key(|c| c.id)
                    .map(|c| c.as_json())
                    .collect::<Vec<_>>();

                let statistic_types = schema::statistic_types::table
                    .filter(schema::statistic_types::placement_type_id.eq(placement_type_id))
                    .load::<db::StatisticType>(conn)?
                    .into_iter()
                    .map(|st| (st.id.unwrap(), st.name))
                    .collect::<BTreeMap<_, _>>();

                let periods = schema::periods::table
                    .filter(schema::periods::statistics_collector_id.eq(collector_id))
                    .load::<db::Period>(conn)?
                    .into_iter()
                    .sorted_by_key(|p| p.id)
                    .map(|p| p.as_json())
                    .collect::<Vec<_>>();

                let stats = schema::statistics::table
                    .filter(schema::statistics::supplier_id.eq(supplier_id))
                    .load::<db::Statistic>(conn)?
                    .into_iter()
                    .group_by(|s| s.statistic_type_id)
                    .into_iter()
                    .map(|(statistic_type_id, stats)| {
                        let statistic_type =
                            statistic_types.get(&statistic_type_id).unwrap().clone();

                        let stats = stats
                            .into_iter()
                            .group_by(|s| s.copy_id)
                            .into_iter()
                            .map(|(copy_id, stats)| {
                                let stats = stats
                                    .into_iter()
                                    .sorted_by_key(|s| s.period_id)
                                    .map(|s| s.value)
                                    .collect::<Vec<_>>();
                                (copy_id, stats)
                            })
                            .sorted_by_key(|(copy_id, _)| *copy_id)
                            .map(|(_, stats)| stats)
                            .collect::<Vec<_>>();
                        (statistic_type, stats)
                    })
                    .collect();

                Ok(json::CollectedStats {
                    copies,
                    periods,
                    stats,
                })
            })
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error::<diesel::result::Error>)?;
    Ok(Json(stats))
}
