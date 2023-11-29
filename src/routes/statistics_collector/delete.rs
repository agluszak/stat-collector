use axum::extract::{Path, State};
use axum::http::StatusCode;
use diesel::prelude::*;

use crate::routes::util::internal_error;
use crate::schema;

/// Deletes a statistics collector
#[utoipa::path(
    delete,
    path = "/statistics_collector/{id}",
    params(
        ("id" = i32, Path, description = "Statistics collector id")
    ),
    responses(
        (status = 200, description = "Ok"),
    )
)]
pub async fn delete_statistics_collector(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(id): Path<i32>,
) -> Result<(), (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    conn.interact(move |conn| {
        diesel::delete(schema::statistics_collectors::table)
            .filter(schema::statistics_collectors::id.eq(id))
            .execute(conn)
            .map_err(internal_error)?;

        Ok(())
    })
    .await
    .map_err(internal_error)??;

    Ok(())
}
