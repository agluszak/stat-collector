use std::sync::{Arc, Mutex};

use crate::logic::email::Mailer;
use axum::extract::FromRef;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::delete;
use axum::{
    routing::{get, post},
    Router,
};
use deadpool_diesel::postgres;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use rust_i18n::{i18n, set_locale};

use tower_http::normalize_path::NormalizePathLayer;

use crate::logic::time::Clock;
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
use crate::routes::supplier::show::__path_show_input_page;
use crate::routes::supplier::show::show_input_page;
use crate::routes::supplier::submit::__path_submit_input;
use crate::routes::supplier::submit::submit_input;

pub mod db;
mod email_templates;
mod errors;
pub mod json;
pub mod logic;
mod routes;
mod schema;

// this embeds the migrations into the application binary
// the migration path is relative to the `CARGO_MANIFEST_DIR`
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("db/migrations/");

#[derive(Debug, Clone)]
pub struct Config {
    pub db_url: String,
    pub smtp_name: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_host: String,
    pub base_url: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_statistics_collector,
        list_statistics_collectors,
        delete_statistics_collector,
        show_statistics_collector,
        get_collector_config,
        show_input_page,
        submit_input,
        send_reminder_emails,
    ),
    components(
        schemas(
            json::sent::Period,
            json::sent::PlacementType,
            json::sent::StatCollector,
            json::sent::Supplier,
            json::received::Period,
            json::received::PlacementType,
            json::received::StatCollector,
            json::received::Supplier,
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
    db_pool: postgres::Pool,
    mailer: Arc<Mutex<dyn Mailer>>,
    clock: Arc<Mutex<dyn Clock>>,
}

impl FromRef<AppState> for deadpool_diesel::postgres::Pool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

impl FromRef<AppState> for Arc<Mutex<dyn Mailer>> {
    fn from_ref(state: &AppState) -> Self {
        state.mailer.clone()
    }
}

impl FromRef<AppState> for Arc<Mutex<dyn Clock>> {
    fn from_ref(state: &AppState) -> Self {
        state.clock.clone()
    }
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Wrong URL")
}

i18n!("locales", fallback = "pl");

pub async fn build_app(
    db_pool: postgres::Pool,
    mailer: Arc<Mutex<dyn Mailer>>,
    clock: Arc<Mutex<dyn Clock>>,
) -> Router {
    set_locale("pl");

    // run the migrations on server startup
    {
        let conn = db_pool.get().await.unwrap();
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
        .route("/statistics_collector/:id", get(show_statistics_collector))
        .route(
            "/statistics_collector/:id",
            delete(delete_statistics_collector),
        )
        .route(
            "/statistics_collector/:id/config",
            get(get_collector_config),
        )
        .route(
            "/statistics_collector/:id/send_emails/:reminder_type",
            post(send_reminder_emails),
        )
        .route("/supplier/:id", get(show_input_page))
        .route("/supplier/:id", post(submit_input))
        .with_state(AppState {
            db_pool,
            mailer,
            clock,
        })
        .fallback(handler_404);

    let collector = collector.layer(NormalizePathLayer::trim_trailing_slash());

    // Slash trailing normalization breaks SwaggerUI, so this must be in a separate router

    Router::new()
        .route("/", get(main_page))
        .merge(collector)
        .merge(docs)
}
