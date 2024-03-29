use dotenvy::dotenv;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use stat_collector::build_app;
use stat_collector::logic::email::AppMailer;
use stat_collector::logic::scheduler::start_scheduler;
use stat_collector::logic::time::AppClock;
use std::env;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::warn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

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

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let smtp_name = env::var("SMTP_NAME").expect("SMTP_NAME must be set");
    let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
    let base_url = env::var("BASE_URL").expect("BASE_URL must be set");

    let mailer = AppMailer::new(
        Mailbox::new(Some(smtp_name), smtp_username.parse().unwrap()),
        &smtp_host,
        587,
        std::time::Duration::from_secs(15),
        Credentials::new(smtp_username, smtp_password),
        &base_url,
    );
    let mailer = Arc::new(Mutex::new(mailer));

    let clock = Arc::new(Mutex::new(AppClock));

    // set up connection pool
    let manager = deadpool_diesel::postgres::Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let db_pool = deadpool_diesel::postgres::Pool::builder(manager)
        .build()
        .unwrap();

    start_scheduler(db_pool.clone(), clock.clone(), mailer.clone())
        .await
        .expect("Failed to start scheduler");

    let app = build_app(db_pool, mailer, clock).await;

    // run it with hyper
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 5433));
    let listener = TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {addr}");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
