use crate::db::StatCollectorId;
use crate::errors::AppError;
use axum::extract::{Path, State};

use diesel::prelude::*;

use crate::schema;

/// Deletes a statistics collector
#[utoipa::path(
    delete,
    path = "/statistics_collector/{id}",
    params(
        ("id" = Uuid, Path, description = "Statistics collector id")
    ),
    responses(
        (status = 200, description = "Ok"),
    )
)]
pub async fn delete_statistics_collector(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(id): Path<StatCollectorId>,
) -> Result<(), AppError> {
    let conn = pool.get().await?;
    conn.interact(move |conn| {
        diesel::delete(schema::statistics_collectors::table)
            .filter(schema::statistics_collectors::id.eq(id))
            .execute(conn)?;

        Ok::<_, diesel::result::Error>(())
    })
    .await??;

    Ok(())
}
