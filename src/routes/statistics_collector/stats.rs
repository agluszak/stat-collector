use crate::routes::util::{internal_error};
use crate::{db, json, schema};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use diesel::prelude::*;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;
use crate::routes::supplier::stats::stats_for_supplier;

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
    Path(collector_id): Path<Uuid>,
) -> Result<Json<BTreeMap<Uuid, json::CollectedStats>>, (StatusCode, String)> {
    // returns Map<CollectorId, CollectedStats>
    let conn = pool.get().await.map_err(internal_error)?;
    let suppliers = conn
        .interact(move |conn| {
                schema::suppliers::table
                    .filter(schema::suppliers::.eq(collector_id))
                    .load::<db::Supplier>(conn)
                    .map_err(internal_error)
            }).await
        .map_err(internal_error)??;

    let mut map = BTreeMap::new();

    for supplier in suppliers {
        let stats = conn.interact(move |conn| {
            stats_for_supplier(conn, supplier.id).map_err(internal_error)
        }).await.map_err(internal_error)??;
        map.insert(supplier.id, stats);
    }

    Ok(Json(map))
}