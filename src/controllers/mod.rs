use chess_module::LichessState;
use games_module::SteamState;
use music_module::SpotifyState;
use search_engine::PostsSearchEngine;
use std::sync::{Arc, OnceLock};

pub mod chess_controller;
pub mod fallback_controller;
pub mod graph_controller;
pub mod metadata_controller;
pub mod music_controller;
pub mod news_controller;
pub mod posts_controller;
pub mod routines_controller;
pub mod steam_controller;

static INDEX_TEMPLATE: OnceLock<String> = OnceLock::new();

pub struct AppState {
    pub app_name: String,
    pub lichess_state: LichessState,
    pub spotify_state: Arc<tokio::sync::Mutex<SpotifyState>>,
    pub steam_state: SteamState,
    pub post_search_engine: Arc<PostsSearchEngine>,
}

pub fn wrap_content_into_full_page(app_name: &str, content: &str) -> String {
    let html = INDEX_TEMPLATE.get_or_init(|| {
        let template_path =
            std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
        std::fs::read_to_string(format!("{}/index.html", template_path))
            .expect("Failed to read index.html template")
    });

    html.replace("{{APPNAME}}", app_name)
        .replace("{{CONTENT}}", content)
}
