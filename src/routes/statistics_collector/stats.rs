use std::collections::BTreeMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;
use itertools::Itertools;

use crate::db::{StatCollectorId, SupplierId};
use crate::routes::supplier::stats::stats_for_supplier;
use crate::routes::util::internal_error;
use crate::{db, json, schema};

/// Gets statistics for a collector
#[utoipa::path(
get,
path = "/statistics_collector/{collector_id}/stats",
params(
("supplier_id" = Uuid, Path, description = "Collector id")
),
responses(
(status = 200, description = "Ok", body = BTreeMap<Uuid, CollectedStats>, content_type = "application/json"),
(status = 404, description = "No such id", content_type = "text/html")
)
)]
pub async fn get_collector_stats(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(collector_id): Path<StatCollectorId>,
) -> Result<Json<BTreeMap<SupplierId, json::CollectedStats>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let map = conn
        .interact(move |conn| {
            let suppliers = schema::suppliers::table
                .select(schema::suppliers::id)
                .filter(
                    schema::suppliers::placement_type_id.eq_any(
                        schema::placement_types::table
                            .select(schema::placement_types::id)
                            .filter(
                                schema::placement_types::statistics_collector_id.eq(collector_id),
                            ),
                    ),
                )
                .load::<db::SupplierId>(conn)?;

            let mut stats = BTreeMap::new();

            for s in suppliers {
                stats.insert(s, stats_for_supplier(conn, s)?);
            }

            Ok::<_, diesel::result::Error>(stats)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?;

    Ok(Json(map))
}
