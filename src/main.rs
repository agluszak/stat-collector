//! Run with
//!
//! ```not_rust
//! cargo run -p example-diesel-postgres
//! ```
//!
//! Checkout the [diesel webpage](https://diesel.rs) for
//! longer guides about diesel
//!
//! Checkout the [crates.io source code](https://github.com/rust-lang/crates.io/)
//! for a real world application using axum and diesel

mod db;
mod schema;
mod json;

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::net::SocketAddr;
use axum::extract::Path;
use maud::{html, Markup};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::schema::statistics_collectors;


// this embeds the migrations into the application binary
// the migration path is relative to the `CARGO_MANIFEST_DIR`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stat-collector=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();

    // set up connection pool
    let manager = deadpool_diesel::postgres::Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let pool = deadpool_diesel::postgres::Pool::builder(manager)
        .build()
        .unwrap();

    // run the migrations on server startup
    {
        let conn = pool.get().await.unwrap();
        conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await
            .unwrap()
            .unwrap();
    }

    // build our application with some routes
    let app = Router::new()
        .route("/statistics_collector/create", post(create_statistics_collector))
        .route("/statistics_collector", get(list_statistics_collectors))
        .route("/statistics_collector/:id", get(show_statistics_collector))
        .with_state(pool);

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 5433));
    tracing::debug!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn create_statistics_collector(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Json(statistics_collector): Json<json::StatisticsCollector>,
) -> Result<(), (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    conn
        .interact(move |conn| {
            conn.transaction(|conn| {
                let db_statistics_collector = db::StatisticsCollector {
                    id: None,
                    name: statistics_collector.name.clone(),
                };
                let collector_id = diesel::insert_into(statistics_collectors::table)
                    .values(&db_statistics_collector)
                    .returning(statistics_collectors::id)
                    .execute(conn)?;

                let db_periods = statistics_collector.periods.iter().map(|period| {
                    db::Period {
                        id: None,
                        name: period.name.clone(),
                        start: period.start_date,
                        end: period.end_date,
                        statistics_collector_id: collector_id as i32,
                    }
                }).collect::<Vec<db::Period>>();

                diesel::insert_into(schema::periods::table)
                    .values(&db_periods)
                    .execute(conn)?;

                let db_placement_types = statistics_collector.placement_types.iter().map(|placement_type| {
                    db::PlacementType {
                        id: None,
                        name: placement_type.name.clone(),
                        statistics_collector_id: collector_id as i32,
                    }
                }).collect::<Vec<db::PlacementType>>();

                let db_placement_types: Vec<db::PlacementType> = diesel::insert_into(schema::placement_types::table)
                    .values(&db_placement_types)
                    .get_results(conn)?;

                let db_suppliers = statistics_collector.placement_types.iter().map(|placement_type| {
                    placement_type.suppliers.iter().map(|supplier| {
                        let placement_type_id = db_placement_types.iter().find(|db_placement_type| db_placement_type.name == placement_type.name).unwrap().id.unwrap();
                        db::Supplier {
                            id: None,
                            name: supplier.name.clone(),
                            mail: supplier.mail.clone(),
                            placement_type_id,
                        }
                    }).collect::<Vec<db::Supplier>>()
                }).flatten().collect::<Vec<db::Supplier>>();

                diesel::insert_into(schema::suppliers::table)
                    .values(&db_suppliers)
                    .execute(conn)?;

                let db_statistic_types = statistics_collector.placement_types.iter().map(|placement_type| {
                    placement_type.statistics.iter().map(|statistic| {
                        let placement_type_id = db_placement_types.iter().find(|db_placement_type| db_placement_type.name == placement_type.name).unwrap().id.unwrap();
                        db::StatisticType {
                            id: None,
                            name: statistic.clone(),
                            placement_type_id,
                        }
                    }).collect::<Vec<db::StatisticType>>()
                }).flatten().collect::<Vec<db::StatisticType>>();

                diesel::insert_into(schema::statistic_types::table)
                    .values(&db_statistic_types)
                    .execute(conn)?;

                let db_copies = statistics_collector.placement_types.iter().map(|placement_type| {
                    placement_type.copies.iter().map(|copy| {
                        let placement_type_id = db_placement_types.iter().find(|db_placement_type| db_placement_type.name == placement_type.name).unwrap().id.unwrap();
                        db::Copy {
                            id: None,
                            name: copy.clone(),
                            placement_type_id,
                        }
                    }).collect::<Vec<db::Copy>>()
                }).flatten().collect::<Vec<db::Copy>>();

                diesel::insert_into(schema::copies::table)
                    .values(&db_copies)
                    .execute(conn)?;

                Ok(())
            })
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error::<diesel::result::Error>)?;
    Ok(())
}

async fn list_statistics_collectors(
    State(pool): State<deadpool_diesel::postgres::Pool>,
) -> Result<Json<Vec<json::StatisticsCollector>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let statistics_collectors = conn
        .interact(|conn| {
            schema::statistics_collectors::table
                .load::<db::StatisticsCollector>(conn)
                .map_err(internal_error)
        })
        .await
        .map_err(internal_error)??;
    let statistics_collectors = statistics_collectors
        .into_iter()
        .map(|statistics_collector| json::StatisticsCollector {
            name: statistics_collector.name,
            periods: vec![],
            placement_types: vec![],
        })
        .collect();
    Ok(Json(statistics_collectors))
}

async fn show_statistics_collector(
    State(pool): State<deadpool_diesel::postgres::Pool>,
    Path(id): Path<i32>,
) -> Result<Markup, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;
    let statistics_collector = conn
        .interact(move |conn| {
            schema::statistics_collectors::table
                .find(id)
                .first::<db::StatisticsCollector>(conn)
                .map_err(internal_error)
        })
        .await
        .map_err(internal_error)??;

    let ok = html! {
        h1 { (statistics_collector.name) }
    };

    Ok(ok)
}

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
    where
        E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
