use crate::db::{StatisticsCollector, Supplier};
use crate::errors::AppError;
use crate::logic::email::{Mailer, ReminderType};
use crate::logic::time::Clock;
use crate::schema;
use chrono::Timelike;
use deadpool_diesel::postgres;
use diesel::prelude::*;
use diesel::{Connection, ExpressionMethods, QueryDsl};
use std::sync::{Arc, Mutex};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use tracing::log;

async fn first_reminder(
    db_pool: postgres::Pool,
    clock: Arc<Mutex<dyn Clock>>,
    mailer: Arc<Mutex<dyn Mailer>>,
) -> Result<(), AppError> {
    // find all collectors that have a period which is due today
    let today = clock.lock().unwrap().now().date_naive();

    let conn = db_pool.get().await?;
    conn.interact(move |conn| {
        conn.transaction(move |conn| {
            let collectors_suppliers = schema::statistics_collectors::table
                .inner_join(schema::periods::table)
                .inner_join(schema::placement_types::table.inner_join(schema::suppliers::table))
                .filter(schema::periods::end.eq(today))
                .select((
                    schema::statistics_collectors::all_columns,
                    schema::suppliers::all_columns,
                ))
                .load::<(StatisticsCollector, Supplier)>(conn)?;

            for (collector, supplier) in collectors_suppliers {
                let address = supplier.mail.parse().unwrap();
                let supplier_id = supplier.id;
                mailer.lock().unwrap().send_reminder(
                    collector,
                    address,
                    supplier_id,
                    ReminderType::FirstReminder,
                )?;
            }

            Ok::<_, AppError>(())
        })
    })
    .await??;

    Ok(())
}

async fn second_reminder(
    db_pool: postgres::Pool,
    clock: Arc<Mutex<dyn Clock>>,
    mailer: Arc<Mutex<dyn Mailer>>,
) -> Result<(), AppError> {
    // find all collectors that have a period which is due today
    // and the last filled date is earlier than today
    let now = clock.lock().unwrap().now();
    let today = now.date_naive();
    let today_9 = now
        .with_hour(9)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();

    let conn = db_pool.get().await?;
    conn.interact(move |conn| {
        conn.transaction(move |conn| {
            let collectors_suppliers = schema::statistics_collectors::table
                .inner_join(schema::periods::table)
                .inner_join(schema::placement_types::table.inner_join(schema::suppliers::table))
                .filter(
                    schema::periods::end
                        .eq(today)
                        .and(schema::suppliers::submitted_date.lt(today_9)),
                )
                .select((
                    schema::statistics_collectors::all_columns,
                    schema::suppliers::all_columns,
                ))
                .load::<(StatisticsCollector, Supplier)>(conn)?;

            for (collector, supplier) in collectors_suppliers {
                let address = supplier.mail.parse().unwrap();
                let supplier_id = supplier.id;
                mailer.lock().unwrap().send_reminder(
                    collector,
                    address,
                    supplier_id,
                    ReminderType::SecondReminder,
                )?;
            }

            Ok::<_, AppError>(())
        })
    })
    .await??;

    Ok(())
}

const FIRST_REMINDER_SCHEDULE: &str = "0 0 8 * * *";
const SECOND_REMINDER_SCHEDULE: &str = "0 0 15 * * *";

pub async fn start_scheduler(
    db_pool: postgres::Pool,
    clock: Arc<Mutex<dyn Clock>>,
    mailer: Arc<Mutex<dyn Mailer>>,
) -> Result<(), JobSchedulerError> {
    let sched = JobScheduler::new().await?;

    {
        let db_pool = db_pool.clone();
        let clock = clock.clone();
        let mailer = mailer.clone();
        sched
            .add(Job::new_async(
                FIRST_REMINDER_SCHEDULE,
                move |_uuid, _l| {
                    let db_pool = db_pool.clone();
                    let clock = clock.clone();
                    let mailer = mailer.clone();
                    Box::pin(async move {
                        first_reminder(db_pool, clock, mailer)
                            .await
                            .unwrap_or_else(|e| {
                                log::error!("Failed to send first reminder: {}", e);
                            });
                    })
                },
            )?)
            .await?;
    }
    {
        let db_pool = db_pool.clone();
        let clock = clock.clone();
        let mailer = mailer.clone();
        sched
            .add(Job::new_async(
                SECOND_REMINDER_SCHEDULE,
                move |_uuid, _l| {
                    let db_pool = db_pool.clone();
                    let clock = clock.clone();
                    let mailer = mailer.clone();
                    Box::pin(async move {
                        second_reminder(db_pool, clock, mailer)
                            .await
                            .unwrap_or_else(|e| {
                                log::error!("Failed to send second reminder: {}", e);
                            });
                    })
                },
            )?)
            .await?;
    }

    sched.shutdown_on_ctrl_c();
    sched.start().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::logic::scheduler::{FIRST_REMINDER_SCHEDULE, SECOND_REMINDER_SCHEDULE};

    #[test]
    fn schedules_can_be_parsed() {
        let _ = tokio_cron_scheduler::JobBuilder::new()
            .with_schedule(FIRST_REMINDER_SCHEDULE)
            .unwrap();
        let _ = tokio_cron_scheduler::JobBuilder::new()
            .with_schedule(SECOND_REMINDER_SCHEDULE)
            .unwrap();
    }
}
