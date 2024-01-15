use std::env;
use std::net::{Ipv4Addr, SocketAddr};

use crate::logic::email::Mailer;
use axum::extract::FromRef;
use axum::routing::delete;
use axum::{
    routing::{get, post},
    Router,
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;

use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use tokio::net::TcpListener;
use tower_http::normalize_path::NormalizePathLayer;
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::main_page;
use crate::routes::statistics_collector::config::__path_get_collector_config;
use crate::routes::statistics_collector::config::get_collector_config;
use crate::routes::statistics_collector::create::__path_create_statistics_collector;
use crate::routes::statistics_collector::create::create_statistics_collector;
use crate::routes::statistics_collector::delete::__path_delete_statistics_collector;
use crate::routes::statistics_collector::delete::delete_statistics_collector;
use crate::routes::statistics_collector::email::__path_send_reminder_emails;
use crate::routes::statistics_collector::email::send_reminder_emails;
use crate::routes::statistics_collector::list::__path_list_statistics_collectors;
use crate::routes::statistics_collector::list::list_statistics_collectors;
use crate::routes::statistics_collector::show::__path_show_statistics_collector;
use crate::routes::statistics_collector::show::show_statistics_collector;
use crate::routes::statistics_collector::stats::__path_get_collector_stats;
use crate::routes::statistics_collector::stats::get_collector_stats;
use crate::routes::supplier::show::__path_show_input_page;
use crate::routes::supplier::show::show_input_page;
use crate::routes::supplier::stats::__path_get_supplier_stats;
use crate::routes::supplier::stats::get_supplier_stats;
use crate::routes::supplier::submit::__path_submit_input;
use crate::routes::supplier::submit::submit_input;

mod db;
mod errors;
mod json;
mod logic;
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
        get_collector_config,
        get_supplier_stats,
        show_input_page,
        submit_input,
        get_collector_stats,
        send_reminder_emails,
    ),
    components(
        schemas(
            json::Period,
            json::PlacementType,
            json::StatisticsCollector,
            json::Supplier,
            json::CollectedStats,
            routes::supplier::submit::FormKey,
            routes::supplier::submit::FormValue,
        )
    ),
    tags(
        (name = "Stat collector")
    )
)]
struct ApiDoc;

#[derive(Clone)]
struct AppState {
    pool: deadpool_diesel::postgres::Pool,
    mailer: Mailer,
}

impl FromRef<AppState> for deadpool_diesel::postgres::Pool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for Mailer {
    fn from_ref(state: &AppState) -> Self {
        state.mailer.clone()
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Wrong URL")
}

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

    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let base_url = env::var("BASE_URL").expect("BASE_URL must be set");

    let mailer = Mailer::new(
        Mailbox::new(Some("StatCollector Reminder".to_string()), smtp_username.parse().unwrap()),
        &smtp_host,
        587,
        std::time::Duration::from_secs(15),
        Credentials::new(
            smtp_username,
            smtp_password,
        ),
        &base_url
    );

    let docs: Router = SwaggerUi::new("/docs")
        .url("/api.json", ApiDoc::openapi())
        .into();

    // build our application with some routes
    let collector = Router::new()
        .route("/statistics_collector", post(create_statistics_collector))
        .route("/statistics_collector", get(list_statistics_collectors))
        .route("/statistics_collector/:id", get(show_statistics_collector))
        .route("/statistics_collector/:id/stats", get(get_collector_stats))
        .route(
            "/statistics_collector/:id",
            delete(delete_statistics_collector),
        )
        .route(
            "/statistics_collector/:id/config",
            get(get_collector_config),
        )
        .route(
            "/statistics_collector/:id/send_emails",
            post(send_reminder_emails),
        )
        .route("/supplier/:id/stats", get(get_supplier_stats))
        .route("/supplier/:id", get(show_input_page))
        .route("/supplier/:id", post(submit_input))
        .with_state(AppState { pool, mailer })
        .fallback(handler_404);

    let collector = collector.layer(NormalizePathLayer::trim_trailing_slash());

    // Slash trailing normalization breaks SwaggerUI, so this must be in a separate router
    let app = Router::new()
        .route("/", get(main_page))
        .merge(collector)
        .merge(docs);

    // run it with hyper
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 5433));
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
