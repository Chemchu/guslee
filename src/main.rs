use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use log::info;
use search_engine::SearchEngine;

use crate::controllers::AppState;

mod controllers;

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
            .service(controllers::posts_controller::landing)
            .service(controllers::news_controller::news_page)
            .service(controllers::metadata_controller::render_metadata)
            .service(controllers::chess_controller::chess_page)
            .service(controllers::chess_controller::chess_graph)
            .service(controllers::posts_controller::search_post)
            .service(controllers::graph_controller::graph_network)
            .service(controllers::graph_controller::garden_view)
            .service(controllers::graph_controller::garden_view_dispatcher)
            .service(controllers::music_controller::get_user_profile)
            .service(controllers::routines_controller::get_current_schedule_activity)
            .service(controllers::posts_controller::get_post_page)
            .service(controllers::posts_controller::fallback_route) // This service should be last because it matches any string
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
