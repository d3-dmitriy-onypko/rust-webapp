use std::{fmt::format, net::TcpListener};

use app::{configuration::get_configuration, startup::run};
use sqlx::{Connection, PgConnection};

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");
    dbg!(&configuration);
    run(
        TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
            .expect("failed to bind"),
        connection,
    )?
    .await
}
