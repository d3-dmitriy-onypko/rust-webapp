use std::net::TcpListener;

use app::run;

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    run(TcpListener::bind("127.0.0.1:0").expect("failed to bind"))?.await
}
