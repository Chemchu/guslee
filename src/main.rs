use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use log::info;
use search_engine::SearchEngine;

use crate::routes::AppState;

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info) // Only show INFO and above (no DEBUG/TRACE)
        .with_module_level("actix_server", log::LevelFilter::Warn) // Only warnings/errors from actix_server
        .with_module_level("actix_web", log::LevelFilter::Info) // Keep actix_web info (for HTTP logs)
        .init()
        .unwrap();

    info!("🚀 Starting server...");

    info!("Creating in-memory full-text search engine...");
    let posts_path = format!("{}/garden", env!("CARGO_MANIFEST_DIR"));
    let search_engine = Arc::new(
        SearchEngine::new(
            &posts_path,
            vec![
                "welcome.md".to_string(),
                "hello.md".to_string(),
                "garden_styling.md".to_string(),
                "journarl/2025/kilbarrack.md".to_string(),
                "journarl/2025/first_job_in_ireland.md".to_string(),
                "journarl/2025/rathmines.md".to_string(),
            ],
        )
        .await,
    );

    info!("Search engine created correctly");
    info!("🌐 Server starting on http://127.0.0.1:3000");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%s %r - %Dms"))
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
