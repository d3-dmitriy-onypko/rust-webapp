use std::{fmt::format, net::TcpListener};

use app::{configuration::get_configuration, startup::run};
use sqlx::{Connection, PgConnection, PgPool};

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("failed to connetect to postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("failed to bind");
    run(listener, db_pool)?.await
}
