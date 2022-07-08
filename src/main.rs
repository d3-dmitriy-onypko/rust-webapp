use std::{fmt::format, net::TcpListener};

use app::{configuration::get_configuration, startup::run};

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    dbg!(&configuration);
    run(
        TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
            .expect("failed to bind"),
    )?
    .await
}
