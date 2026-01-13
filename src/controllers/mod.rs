use search_engine::SearchEngine;
use std::sync::OnceLock;

pub mod chess_controller;
pub mod graph_controller;
pub mod metadata_controller;
pub mod music_controller;
pub mod news_controller;
pub mod posts_controller;
pub mod routines_controller;

static INDEX_TEMPLATE: OnceLock<String> = OnceLock::new();

pub struct AppState {
    pub app_name: String,
    pub lichess_state: LichessState,
    pub spotify_state: SpotifyState,
    pub search_engine: std::sync::Arc<SearchEngine>,
}

pub struct LichessState {
    pub lichess_token: String,
    pub lichess_username: String,
}

pub struct SpotifyState {
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
    pub spotify_session: (String, i64), // (token, expiration in secs)
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

pub fn load_html_page(html_file: &str) -> String {
    let template_path =
        std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());

    std::fs::read_to_string(format!("{}/{}.html", template_path, html_file))
        .expect("Failed to read chess_page.html template")
}
