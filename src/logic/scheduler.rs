use std::sync::{Arc, Mutex};
use deadpool_diesel::postgres;
use diesel::{Connection, ExpressionMethods, QueryDsl};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use crate::errors::AppError;
use crate::logic::time::Clock;
use crate::schema;
use diesel::prelude::*;
use tracing::log;
use crate::db::{StatCollectorId, StatisticsCollector, Supplier};
use crate::logic::email::{Mailer, ReminderType};


async fn first_reminder(db_pool: postgres::Pool, clock: Arc<Mutex<dyn Clock>>, mailer: Arc<Mutex<dyn Mailer>>) -> Result<(), AppError> {
    // find all collectors that have a period which is due today
    let today = clock.lock().unwrap().now().date();

    let conn = db_pool.get().await?;
    conn.interact(move |conn| {
        conn.transaction(move |conn| {
            let collectors_suppliers = schema::statistics_collectors::table
                .inner_join(schema::periods::table)
                .inner_join(schema::placement_types::table.inner_join(schema::suppliers::table))
                .filter(schema::periods::end.eq(today))
                .select((schema::statistics_collectors::all_columns, schema::suppliers::all_columns))
                .load::<(StatisticsCollector, Supplier)>(conn)?;

            for (collector, supplier) in collectors_suppliers {
                let address = supplier.mail.parse().unwrap();
                let supplier_id = supplier.id;
                mailer.lock().unwrap().send_reminder(collector, address, supplier_id, ReminderType::FirstReminder)?;
            }


            Ok::<_, AppError>(())
        })
    }).await??;

    Ok(())
}

async fn start_scheduler(db_pool: postgres::Pool, clock: Arc<Mutex<dyn Clock>>, mailer: Arc<Mutex<dyn Mailer>>) -> Result<(), JobSchedulerError> {
    let sched = JobScheduler::new().await?;

    sched.add(
        Job::new_async("0 9 * * *", move |uuid, mut l| {
            let db_pool = db_pool.clone();
            let clock = clock.clone();
            let mailer = mailer.clone();
            Box::pin(async move {
                first_reminder(db_pool, clock, mailer).await.unwrap_or_else(|e| {
                    log::error!("Failed to send first reminder: {}", e);
                });
            })
        })?
    ).await?;

    sched.shutdown_on_ctrl_c();
    sched.start().await?;

    Ok(())
}