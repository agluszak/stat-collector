use crate::db::SupplierId;
use crate::routes::util::internal_error;
use crate::{db, json, schema};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;

use itertools::Itertools;
use std::collections::BTreeMap;

pub fn stats_for_supplier(
    conn: &mut PgConnection,
    supplier_id: db::SupplierId,
) -> Result<json::CollectedStats, diesel::result::Error> {
    conn.transaction(|conn| {
        let placement_type_id = schema::suppliers::table
            .inner_join(schema::placement_types::table)
            .filter(schema::suppliers::id.eq(supplier_id))
            .select(schema::placement_types::id)
            .first::<db::PlacementTypeId>(conn)?;

        let collector_id = schema::placement_types::table
            .filter(schema::placement_types::id.eq(placement_type_id))
            .select(schema::placement_types::statistics_collector_id)
            .first::<db::StatCollectorId>(conn)?;

        let copies = schema::copies::table
            .filter(schema::copies::placement_type_id.eq(placement_type_id))
            .load::<db::Copy>(conn)?;

        let periods = schema::periods::table
            .filter(schema::periods::statistics_collector_id.eq(collector_id))
            .load::<db::Period>(conn)?;

        let statistic_types = schema::statistic_types::table
            .filter(schema::statistic_types::placement_type_id.eq(placement_type_id))
            .load::<db::StatisticType>(conn)?;

        let stats = schema::statistics::table
            .filter(schema::statistics::supplier_id.eq(supplier_id))
            .load::<db::Statistic>(conn)?
            .into_iter()
            .group_by(|s| s.statistic_type_id)
            .into_iter()
            .map(|(statistic_type_id, stats)| {
                let statistic_type = statistic_types
                    .iter()
                    .find(|s| s.id == statistic_type_id)
                    .unwrap();

                let stats = stats
                    .into_iter()
                    .group_by(|s| s.copy_id)
                    .into_iter()
                    .sorted_by_key(|(copy_id, _)| *copy_id)
                    .map(|(copy_id, stats)| {
                        let stats = stats
                            .into_iter()
                            .sorted_by_key(|s| s.period_id)
                            .map(|s| s.value)
                            .collect::<Vec<_>>();
                        (copy_id, stats)
                    })
                    .map(|(_, stats)| stats)
                    .collect_vec();
                (statistic_type.name.clone(), stats)
            })
            .collect();

        let collected_stats = json::CollectedStats {
            periods: periods
                .iter()
                .sorted_by_key(|p| p.id)
                .map(|p| p.as_json())
                .collect(),
            copies: copies
                .iter()
                .sorted_by_key(|c| c.id)
                .map(|c| c.as_json())
                .collect(),
            stats,
        };

        Ok(collected_stats)
    })
}

/// Gets statistics for a supplier
#[utoipa::path(
    get,
    path = "/supplier/{supplier_id}/stats",
    params(
        ("supplier_id" = Uuid, Path, description = "Supplier id")
    ),
    responses(
        (status = 200, description = "Ok", body = CollectedStats, content_type = "application/json"),
        (status = 404, description = "No such id", content_type = "text/html")
    )
)]
pub async fn get_supplier_stats(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(supplier_id): Path<SupplierId>,
) -> Result<Json<json::CollectedStats>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let stats = conn
        .interact(move |conn| stats_for_supplier(conn, supplier_id).map_err(internal_error))
        .await
        .map_err(internal_error)?
        .ok();

    if let Some(stats) = stats {
        Ok(Json(stats))
    } else {
        Err((StatusCode::NOT_FOUND, "No such id".to_string()))
    }
}
