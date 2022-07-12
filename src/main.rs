use std::net::TcpListener;

use app::{configuration::get_configuration, startup::run, telemetry::*};
use sqlx::PgPool;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration");
    let db_pool = PgPool::connect_lazy(&configuration.database.connection_string())
        .expect("failed to connetect to postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).expect("failed to bind");
    run(listener, db_pool)?.await
}
