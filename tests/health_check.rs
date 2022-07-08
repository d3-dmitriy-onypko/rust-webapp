//! tests/health_check.rs

use std::net::TcpListener;

use app::configuration::{get_configuration, Settings};
use reqwest::Client;
use sqlx::{Connection, PgConnection, PgPool};
use tokio::runtime::Runtime;
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
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = listener.local_addr().unwrap().port();
    let connection_pool = PgPool::connect(&settings.database.connection_string())
        .await
        .expect("connected");
    let server = app::startup::run(listener, connection_pool).expect("Failed to bind to address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
