use std::sync::Arc;

use actix_web::{App, HttpServer, middleware::Logger, web};
use log::info;
use search_engine::PostsSearchEngine;

use crate::controllers::AppState;

mod controllers;
mod helpers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info) // Only show INFO and above (no DEBUG/TRACE)
        .with_module_level("actix_server", log::LevelFilter::Warn) // Only warnings/errors from actix_server
        .with_module_level("actix_web", log::LevelFilter::Info) // Keep actix_web info (for HTTP logs)
        .init()
        .unwrap();

    info!("Starting server...");

    info!("Loading environment variables...");
    let env_vars = helpers::read_env_file();

    let lichess_token = env_vars
        .get("LICHESS_API_TOKEN")
        .expect("LICHESS_API_TOKEN not defined")
        .to_string();

    let lichess_username = env_vars
        .get("LICHESS_USERNAME")
        .expect("LICHESS_USERNAME not defined")
        .to_string();

    let spotify_user_id = env_vars
        .get("SPOTIFY_USER_ID")
        .expect("SPOTIFY_USER_ID not defined")
        .to_string();

    let spotify_client_id = env_vars
        .get("SPOTIFY_CLIENT_ID")
        .expect("SPOTIFY_CLIENT_ID not defined")
        .to_string();

    let spotify_client_secret = env_vars
        .get("SPOTIFY_CLIENT_SECRET")
        .expect("SPOTIFY_CLIENT_SECRET not defined")
        .to_string();

    let spotify_refresh_token = env_vars
        .get("SPOTIFY_REFRESH_TOKEN")
        .expect("SPOTIFY_REFRESH_TOKEN not defined")
        .to_string();

    let steam_token = env_vars
        .get("STEAM_API_KEY")
        .expect("STEAM_API_KEY not defined")
        .to_string();

    info!("Creating in-memory full-text search engine...");
    let search_engine = Arc::new(PostsSearchEngine::new("./garden").await);

    info!("Initializing Spotify state...");
    let spotify_state = Arc::new(tokio::sync::Mutex::new(
        music_module::SpotifyState::from_refresh_token(
            spotify_user_id,
            spotify_client_id,
            spotify_client_secret,
            spotify_refresh_token,
        ),
    ));
    info!("Spotify state initialized");

    info!("Search engine created correctly");
    info!("Server starting on port 3000");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%s %r - %Dms"))
            .service(actix_files::Files::new("/_static", "./static").show_files_listing())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Gus' digital garden"),
                lichess_state: chess_module::LichessState {
                    lichess_token: lichess_token.clone(),
                    lichess_username: lichess_username.clone(),
                },
                spotify_state: Arc::clone(&spotify_state),
                steam_state: steam_token.clone(),
                post_search_engine: Arc::clone(&search_engine),
            }))
            .service(controllers::posts_controller::landing)
            .service(controllers::news_controller::news_page)
            .service(controllers::metadata_controller::render_metadata)
            .service(controllers::steam_controller::steam_page)
            .service(controllers::chess_controller::chess_page)
            .service(controllers::chess_controller::chess_graph)
            .service(controllers::posts_controller::search_post)
            .service(controllers::graph_controller::graph_network)
            .service(controllers::graph_controller::garden_view)
            .service(controllers::graph_controller::garden_view_dispatcher)
            .service(controllers::music_controller::get_user_profile)
            .service(controllers::music_controller::get_user_top_item)
            .service(controllers::routines_controller::get_current_schedule_activity)
            .service(controllers::posts_controller::get_post_page)
            .service(controllers::posts_controller::fallback_route) // This service should be last one in the list because it matches any string
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
