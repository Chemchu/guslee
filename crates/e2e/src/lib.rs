use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use actix_web::{HttpServer, web};
use guslee::controllers::AppState;
use search_engine::PostsSearchEngine;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use wiremock::MockServer;

pub mod mocks;

pub const APP_NAME: &str = "E2E garden";
pub const REPO_SOURCE: &str = "https://github.com/example/garden/blob/main";
pub const LICHESS_USERNAME: &str = "FixturePlayer";
pub const LICHESS_TOKEN: &str = "lichess-token";
pub const SPOTIFY_REFRESH_TOKEN: &str = "spotify-refresh-token";
pub const STEAM_API_KEY: &str = "steam-key";
pub const STEAM_ID: &str = "76561190000000000";

pub const GARDEN_PATH: &str = "crates/e2e/fixtures/garden";
pub const REAL_GARDEN_PATH: &str = "garden";

pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
    pub lichess: MockServer,
    pub spotify: MockServer,
    pub steam: MockServer,
}

pub struct Response {
    pub status: u16,
    pub headers: reqwest::header::HeaderMap,
    pub body: String,
}

impl Response {
    pub fn is_full_page(&self) -> bool {
        self.body.starts_with("<!doctype html>")
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }
}

fn enter_workspace_root() {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    let root = ROOT.get_or_init(|| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace root")
    });
    std::env::set_current_dir(root).expect("enter workspace root");
}

impl TestApp {
    pub async fn spawn() -> TestApp {
        TestApp::spawn_with_garden(GARDEN_PATH).await
    }

    pub async fn spawn_with_garden(garden_path: &str) -> TestApp {
        enter_workspace_root();

        let lichess = MockServer::start().await;
        let spotify = MockServer::start().await;
        let steam = MockServer::start().await;

        let search_engine =
            PostsSearchEngine::new_with_endpoint("mem://", REPO_SOURCE, garden_path).await;

        let app_state = web::Data::new(AppState {
            app_name: APP_NAME.to_string(),
            garden_path: garden_path.to_string(),
            lichess_state: chess_module::LichessState::new(
                LICHESS_TOKEN.to_string(),
                LICHESS_USERNAME.to_string(),
            )
            .with_api_base_url(lichess.uri()),
            spotify_state: Arc::new(tokio::sync::Mutex::new(
                music_module::SpotifyState::from_refresh_token(
                    "spotify-user".to_string(),
                    "spotify-client".to_string(),
                    "spotify-secret".to_string(),
                    SPOTIFY_REFRESH_TOKEN.to_string(),
                )
                .with_base_urls(spotify.uri(), spotify.uri()),
            )),
            steam_state: games_module::SteamState::new(
                STEAM_API_KEY.to_string(),
                STEAM_ID.to_string(),
            )
            .with_api_base_url(steam.uri()),
            post_search_engine: Arc::new(search_engine),
        });

        let server = HttpServer::new(move || guslee::create_app(app_state.clone()))
            .workers(1)
            .bind(("127.0.0.1", 0))
            .expect("bind random port");
        let port = server.addrs()[0].port();
        tokio::spawn(server.run());

        TestApp {
            address: format!("http://127.0.0.1:{port}"),
            client: reqwest::Client::new(),
            lichess,
            spotify,
            steam,
        }
    }

    pub async fn get(&self, path: &str) -> Response {
        self.send(self.client.get(self.url(path))).await
    }

    pub async fn htmx_get(&self, path: &str) -> Response {
        self.send(
            self.client
                .get(self.url(path))
                .header("HX-Request", "true")
                .header("HX-Request-Type", "partial"),
        )
        .await
    }

    pub async fn history_restore_get(&self, path: &str) -> Response {
        self.send(
            self.client
                .get(self.url(path))
                .header("HX-Request", "true")
                .header("HX-Request-Type", "full")
                .header("HX-History-Restore-Request", "true"),
        )
        .await
    }

    pub async fn send(&self, request: reqwest::RequestBuilder) -> Response {
        let response = request.send().await.expect("request failed");
        Response {
            status: response.status().as_u16(),
            headers: response.headers().clone(),
            body: response.text().await.expect("response body"),
        }
    }

    pub async fn raw_get(&self, raw_path: &str) -> (u16, String) {
        let host = self.address.trim_start_matches("http://");
        let mut stream = tokio::net::TcpStream::connect(host).await.expect("connect");
        let request =
            format!("GET {raw_path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.expect("write");

        let mut raw = String::new();
        stream.read_to_string(&mut raw).await.expect("read");
        let status = raw
            .split_whitespace()
            .nth(1)
            .and_then(|code| code.parse().ok())
            .expect("status code");
        let body = raw
            .split_once("\r\n\r\n")
            .map(|(_, body)| body.to_string())
            .unwrap_or_default();
        (status, body)
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }
}
