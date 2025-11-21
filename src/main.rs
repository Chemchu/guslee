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
    let garden_path = std::env::var("GARDEN_PATH")
        .unwrap_or_else(|_| format!("{}/garden", env!("CARGO_MANIFEST_DIR")));
    let search_engine = Arc::new(SearchEngine::new(&garden_path).await);

    info!("Search engine created correctly");
    info!("🌐 Server starting on http://0.0.0.0:3000");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%s %r - %Dms"))
            .service(actix_files::Files::new("/_static", "./static").show_files_listing())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Gus' digital garden"),
                garden_path: garden_path.clone(),
                search_engine: Arc::clone(&search_engine),
            }))
            .service(routes::landing)
            .service(routes::news_routes::news_page)
            .service(routes::metadata_routes::render_metadata)
            .service(routes::chess_routes::chess_page)
            .service(routes::chess_routes::chess_graph)
            .service(routes::posts_routes::search_post)
            .service(routes::graph_routes::graph_network)
            .service(routes::routines_routes::get_current_schedule_activity)
            .service(routes::posts_routes::get_post)
            .service(routes::posts_routes::fallback_route) // This service should be last because it matches any string
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
