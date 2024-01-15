use axum::{extract::State, response::Json};
use diesel::prelude::*;

use crate::errors::AppError;
use crate::{db, schema};

/// Lists all statistics collectors
#[utoipa::path(
    get,
    path = "/statistics_collector",
    responses(
        (status = 200, description = "Ok"),
    )
)]
pub async fn list_statistics_collectors(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<db::StatisticsCollector>>, AppError> {
    let conn = pool.get().await?;
    let statistics_collectors = conn
        .interact(|conn| schema::statistics_collectors::table.load::<db::StatisticsCollector>(conn))
        .await??;
    Ok(Json(statistics_collectors))
}
