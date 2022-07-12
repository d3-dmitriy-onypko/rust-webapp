//! tests/health_check.rs

use std::net::TcpListener;

use app::{
    configuration::{get_configuration, DatabaseSettings, Settings},
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use reqwest::Client;
use sqlx::{Connection, Executor, PgConnection, PgPool};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let WebTest {
        address,
        client,
        settings,
    } = WebTest::new().await;

    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    //
    // Use `cargo add reqwest --dev --vers 0.11` to add // it under `[dev-dependencies]` in Cargo.toml
    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_400_for_valid_for_data() {
    let WebTest {
        address,
        client,
        settings,
    } = WebTest::new().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The APi did not fail with 400 Bad Request when payload was {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_for_data() {
    let WebTest {
        address,
        client,
        settings,
    } = WebTest::new().await;
    let connection_string = settings.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to db");

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

struct WebTest {
    address: String,
    client: Client,
    settings: Settings,
}

impl WebTest {
    async fn new() -> WebTest {
        let configuration = get_configuration().expect("failed to read config");
        WebTest {
            address: spawn_app(&configuration).await,
            client: reqwest::Client::new(),
            settings: configuration,
        }
    }
}

// Launch our application in the background ~somehow~
async fn spawn_app(settings: &Settings) -> String {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = listener.local_addr().unwrap().port();
    let mut settings = settings.to_owned();
    settings.database.database_name = uuid::Uuid::new_v4().to_string();
    let connection_pool = configure_database(&settings.database).await;
    let server = app::startup::run(listener, connection_pool).expect("Failed to bind to address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("failed to connect");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create db");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("connected");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("migration failed");

    connection_pool
}
