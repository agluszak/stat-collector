use axum_test::TestServer;

use stat_collector::db::StatCollectorId;
use stat_collector::logic::email::MockMailer;
use stat_collector::logic::email::ReminderType::{FirstReminder, SecondReminder};
use stat_collector::logic::time::AppClock;
use stat_collector::{build_app, db, json};
use std::sync::{Arc, Mutex};
use testcontainers_modules::{postgres::Postgres, testcontainers::clients::Cli};
use time::{Date, Month};
use uuid::Uuid;

#[tokio::test]
async fn main() {
    // startup the module
    let docker = Cli::default();
    let node = docker.run(Postgres::default());

    // prepare connection string
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        node.get_host_port_ipv4(5432)
    );

    let mailer = Arc::new(Mutex::new(MockMailer::new()));
    let clock = Arc::new(Mutex::new(AppClock));

    let app = build_app(connection_string, mailer.clone(), clock.clone()).await;

    let server = TestServer::new(app).unwrap();

    let new_collector = json::received::StatCollector {
        name: "kolektor testowy".to_string(),
        client: "pepsi".to_string(),
        periodicity: "tygodniowo".to_string(),
        weekday: "Wednesday".to_string(),
        periods: vec![
            json::received::Period {
                name: "2023.11.08 - 11.14".to_string(),
                start_date: Date::from_calendar_date(2023, Month::November, 8).unwrap(),
                end_date: Date::from_calendar_date(2023, Month::November, 14).unwrap(),
            },
            json::received::Period {
                name: "2023.11.15 - 11.21".to_string(),
                start_date: Date::from_calendar_date(2023, Month::November, 15).unwrap(),
                end_date: Date::from_calendar_date(2023, Month::November, 21).unwrap(),
            },
            json::received::Period {
                name: "2023.11.22 - 11.28".to_string(),
                start_date: Date::from_calendar_date(2023, Month::November, 22).unwrap(),
                end_date: Date::from_calendar_date(2023, Month::November, 28).unwrap(),
            },
        ],
        placement_types: vec![
            json::received::PlacementType {
                name: "Display".to_string(),
                suppliers: vec![json::received::Supplier {
                    name: "Google".to_string(),
                    mail: "google@google.com".parse().unwrap(),
                }],
                statistics: vec!["Conversions".to_string()],
                copies: vec!["kopia a".to_string(), "kopia b".to_string()],
            },
            json::received::PlacementType {
                name: "Mailing".to_string(),
                suppliers: vec![
                    json::received::Supplier {
                        name: "Inis".to_string(),
                        mail: "inis@inis.com".parse().unwrap(),
                    },
                    json::received::Supplier {
                        name: "Inis2".to_string(),
                        mail: "inis2@inis.com".parse().unwrap(),
                    },
                ],
                statistics: vec!["Impressions".to_string()],
                copies: vec!["kopia c".to_string()],
            },
        ],
    };

    let response = server
        .post("/statistics_collector")
        .json(&new_collector)
        .await;

    response.assert_status_ok();
    let id: Uuid = response.json();

    let response = server.get("/statistics_collector").await;
    response.assert_status_ok();

    // dbg!(response.text());

    let collectors = response.json::<Vec<db::StatisticsCollector>>();
    assert_eq!(collectors.len(), 1);

    let collector = &collectors[0];

    assert_eq!(collector.id, StatCollectorId::from(id));
    assert_eq!(collector.name, new_collector.name);
    assert_eq!(collector.client, new_collector.client);
    assert_eq!(collector.periodicity, new_collector.periodicity);
    assert_eq!(collector.weekday, new_collector.weekday);

    let response = server
        .get(&format!("/statistics_collector/{}/config", id))
        .await;

    // dbg!(response.text());

    response.assert_status_ok();

    let collector = response.json::<json::sent::StatCollector>();

    assert_eq!(collector.name, new_collector.name);
    assert_eq!(collector.client, new_collector.client);
    assert_eq!(collector.periodicity, new_collector.periodicity);
    assert_eq!(collector.weekday, new_collector.weekday);
    assert_eq!(collector.periods.len(), new_collector.periods.len());
    assert_eq!(
        collector.placement_types.len(),
        new_collector.placement_types.len()
    );
    assert_eq!(collector.id, StatCollectorId::from(id));

    mailer
        .lock()
        .unwrap()
        .expect_send_reminder()
        .withf(move |_, _, _, reminder_type| *reminder_type == FirstReminder)
        .times(3)
        .returning(|_, _, _, _| Ok(()));

    // Test manual email sending
    let response = server
        .post(&format!(
            "/statistics_collector/{}/send_emails/FirstReminder",
            id
        ))
        .await;

    response.assert_status_ok();

    mailer.lock().unwrap().checkpoint();

    mailer
        .lock()
        .unwrap()
        .expect_send_reminder()
        .withf(move |_, _, _, reminder_type| *reminder_type == SecondReminder)
        .times(3)
        .returning(|_, _, _, _| Ok(()));

    let response = server
        .post(&format!(
            "/statistics_collector/{}/send_emails/SecondReminder",
            id
        ))
        .await;

    response.assert_status_ok();
}
