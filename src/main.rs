use std::sync::Arc;

use actix_web::{HttpServer, web};
use guslee::{controllers::AppState, create_app, helpers};
use log::info;
use search_engine::PostsSearchEngine;

const GARDEN_PATH: &str = "./garden";

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
    let env_var = |name: &str| env_vars[name].clone();

    info!("Initializing Lichess, Spotify and Steam state...");
    let lichess_state =
        chess_module::LichessState::new(env_var("LICHESS_API_TOKEN"), env_var("LICHESS_USERNAME"));

    let spotify_state = Arc::new(tokio::sync::Mutex::new(
        music_module::SpotifyState::from_refresh_token(
            env_var("SPOTIFY_USER_ID"),
            env_var("SPOTIFY_CLIENT_ID"),
            env_var("SPOTIFY_CLIENT_SECRET"),
            env_var("SPOTIFY_REFRESH_TOKEN"),
        ),
    ));

    let steam_state = games_module::SteamState::new(env_var("STEAM_API_KEY"), env_var("STEAM_ID"));

    info!("Creating in-memory full-text search engine...");
    let search_engine =
        Arc::new(PostsSearchEngine::new(env_var("REPO_SOURCE").as_str(), GARDEN_PATH).await);
    info!("Search engine created correctly");

    let app_state = web::Data::new(AppState {
        app_name: String::from("Gus' digital garden"),
        garden_path: GARDEN_PATH.to_string(),
        lichess_state,
        spotify_state,
        steam_state,
        post_search_engine: search_engine,
    });

    info!("Server starting on port 3000");

    HttpServer::new(move || create_app(app_state.clone()))
        .bind(("0.0.0.0", 3000))?
        .run()
        .await
}
