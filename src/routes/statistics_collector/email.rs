use axum::extract::Path;
use axum::extract::State;
use diesel::prelude::*;
use std::sync::Arc;

use crate::db::StatCollectorId;

use crate::errors::AppError;
use crate::logic::email::Mailer;
use crate::{db, schema};

/// Sends reminder emails to all suppliers of a statistics collector
#[utoipa::path(
    post,
    path = "/statistics_collector/{id}/send_emails",
    params(
        ("id" = Uuid, Path, description = "Statistics collector id")
    ),
    responses(
    (status = 200, description = "Ok"),
    )
)]
pub async fn send_reminder_emails(
    State(mailer): State<Arc<dyn Mailer>>,
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(id): Path<StatCollectorId>,
) -> Result<(), AppError> {
    let conn = pool.get().await?;
    conn.interact(move |conn| {
        conn.transaction(move |conn| {
            let stat_collector = schema::statistics_collectors::table
                .find(id)
                .first::<db::StatisticsCollector>(conn)
                .map_err(|_| AppError::not_found("statistics collector", id))?;

            let suppliers: Vec<db::Supplier> = schema::placement_types::table
                .filter(schema::placement_types::statistics_collector_id.eq(id))
                .inner_join(schema::suppliers::table)
                .select(db::Supplier::as_select())
                .load(conn)?;

            for supplier in suppliers {
                mailer.send_reminder(
                    stat_collector.clone(),
                    supplier.mail.parse().unwrap(),
                    supplier.id,
                )?;
            }

            Ok::<(), AppError>(())
        })
    })
    .await??;
    Ok(())
}
