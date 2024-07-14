use actix_files as fs;
use actix_web::{App, HttpServer};

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/_static", "static").show_files_listing())
            .service(routes::landing_page)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
