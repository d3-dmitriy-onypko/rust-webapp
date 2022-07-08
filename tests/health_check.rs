//! tests/health_check.rs

use std::net::TcpListener;

use reqwest::Client;
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();

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
    let WebTest { address, client } = WebTest::new();

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
        
        assert_eq!(400, response.status().as_u16(),
    "The APi did not fail with 400 Bad Request when payload was {}", error_message)
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_for_data() {
    let WebTest { address, client } = WebTest::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

struct WebTest {
    address: String,
    client: Client,
}

impl WebTest {
    fn new() -> WebTest {
        WebTest {
            address: spawn_app(),
            client: reqwest::Client::new(),
        }
    }
}

// Launch our application in the background ~somehow~
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind");
    let port = listener.local_addr().unwrap().port();
    let server = app::run(listener).expect("Failed to bind to address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
