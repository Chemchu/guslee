use actix_web::{App, HttpServer};

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(actix_files::Files::new("/_static", "./static").show_files_listing())
            .service(routes::landing)
            .service(routes::post)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
