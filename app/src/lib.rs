use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, dev::Server};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .service(greet)
    })
    .bind(("127.0.0.1", 8080))?
    .run();

    Ok(server)
}