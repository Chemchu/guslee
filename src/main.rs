use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use search_engine::SearchEngine;

use crate::routes::AppState;

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let posts_path = "/garden";
    let search_engine = Arc::new(SearchEngine::new(
        format!("{}{}", env!("CARGO_MANIFEST_DIR"), posts_path).as_str(),
    ));

    HttpServer::new(move || {
        App::new()
            .service(actix_files::Files::new("/_static", "./static").show_files_listing())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Gustavo's digital garden"),
                search_engine: Arc::clone(&search_engine),
            }))
            .service(routes::landing)
            .service(routes::search_post)
            .service(routes::post)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
