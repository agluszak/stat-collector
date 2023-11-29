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

use std::net::{Ipv4Addr, SocketAddr};

use axum::routing::delete;
use axum::{
    routing::{get, post},
    Router,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use tower_http::normalize_path::NormalizePathLayer;
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::input::show::__path_show_input_page;
use crate::routes::input::show::show_input_page;
use crate::routes::input::submit::__path_submit_input;
use crate::routes::input::submit::submit_input;
use crate::routes::main_page;
use crate::routes::statistics_collector::create::__path_create_statistics_collector;
use crate::routes::statistics_collector::create::create_statistics_collector;
use crate::routes::statistics_collector::delete::__path_delete_statistics_collector;
use crate::routes::statistics_collector::delete::delete_statistics_collector;
use crate::routes::statistics_collector::list::__path_list_statistics_collectors;
use crate::routes::statistics_collector::list::list_statistics_collectors;
use crate::routes::statistics_collector::show::__path_show_statistics_collector;
use crate::routes::statistics_collector::show::show_statistics_collector;
use crate::routes::statistics_collector::stats::__path_get_stats;
use crate::routes::statistics_collector::stats::get_stats;

mod db;
mod json;
mod render_html;
mod routes;
mod schema;

// this embeds the migrations into the application binary
// the migration path is relative to the `CARGO_MANIFEST_DIR`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/migrations/");

#[derive(OpenApi)]
#[openapi(
    paths(
        create_statistics_collector,
        list_statistics_collectors,
        delete_statistics_collector,
        show_statistics_collector,
        get_stats,
        show_input_page,
        submit_input,
    ),
    components(
        schemas(json::Period, json::PlacementType, json::StatisticsCollector, json::Supplier, json::CollectedStats)
    ),
    tags(
        (name = "Stat collector")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    if let Err(e) = dotenv() {
        warn!("Failed to load .env file: {}", e);
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stat-collector=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

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

    let docs: Router = SwaggerUi::new("/docs")
        .url("/api.json", ApiDoc::openapi())
        .into();

    // build our application with some routes
    let collector = Router::new()
        .route("/statistics_collector", post(create_statistics_collector))
        .route("/statistics_collector", get(list_statistics_collectors))
        .route("/statistics_collector", delete(delete_statistics_collector))
        .route("/statistics_collector/:id", get(show_statistics_collector))
        .route("/statistics_collector/:id/stats", get(get_stats))
        .route("/input/:id", get(show_input_page))
        .route("/input/:id", post(submit_input))
        .with_state(pool);

    let collector = collector.layer(NormalizePathLayer::trim_trailing_slash());

    // Slash trailing normalization breaks SwaggerUI, so this must be in a separate router
    let app = Router::new()
        .route("/", get(main_page))
        .merge(collector)
        .merge(docs);

    // run it with hyper
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 5433));
    tracing::debug!("listening on {addr}");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
