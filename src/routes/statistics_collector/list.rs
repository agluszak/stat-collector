use axum::{extract::State, http::StatusCode, response::Json};
use diesel::prelude::*;

use crate::routes::util::internal_error;
use crate::{db, schema};

pub async fn list_statistics_collectors(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<db::StatisticsCollector>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let statistics_collectors = conn
        .interact(|conn| {
            schema::statistics_collectors::table
                .load::<db::StatisticsCollector>(conn)
                .map_err(internal_error)
        })
        .await
        .map_err(internal_error)??;
    Ok(Json(statistics_collectors))
}
