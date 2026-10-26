use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web::Html};
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
pub mod tags_controller;

static INDEX_TEMPLATE: OnceLock<String> = OnceLock::new();

pub struct AppState {
    pub app_name: String,
    pub garden_path: String,
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

pub fn is_partial_request(req: &HttpRequest) -> bool {
    let headers = req.headers();
    let is_htmx_req = headers.contains_key("HX-Request");
    let is_history_restore = headers.contains_key("HX-History-Restore-Request");
    let is_full_swap = headers
        .get("HX-Request-Type")
        .is_some_and(|request_type| request_type == "full");

    is_htmx_req && !is_history_restore && !is_full_swap
}

pub fn render_page(app_state: &AppState, req: &HttpRequest, content: &str) -> Html {
    if is_partial_request(req) {
        Html::new(content.to_string())
    } else {
        Html::new(wrap_content_into_full_page(&app_state.app_name, content))
    }
}

pub fn render_not_found(app_state: &AppState, req: &HttpRequest) -> HttpResponse {
    let content = maud::html! {
        div class="flex flex-col w-full gap-4 p-6 lg:p-8" {
            h1 class="text-4xl text-primary-color" { "404" }
            p { "This page does not exist (yet)." }
            a href="/posts/welcome" hx-target="#main-section" class="hover:text-primary-color underline" {
                "Back to the garden"
            }
        }
    }
    .into_string();

    let body = if is_partial_request(req) {
        content
    } else {
        wrap_content_into_full_page(&app_state.app_name, &content)
    };

    HttpResponse::NotFound()
        .content_type(ContentType::html())
        .body(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn plain_browser_requests_need_the_full_page() {
        let req = TestRequest::get().to_http_request();
        assert!(!is_partial_request(&req));
    }

    #[test]
    fn htmx_requests_targeting_a_section_are_partial() {
        let req = TestRequest::get()
            .insert_header(("HX-Request", "true"))
            .insert_header(("HX-Request-Type", "partial"))
            .to_http_request();
        assert!(is_partial_request(&req));

        let req = TestRequest::get()
            .insert_header(("HX-Request", "true"))
            .to_http_request();
        assert!(is_partial_request(&req));
    }

    #[test]
    fn htmx_body_swaps_need_the_full_page() {
        let req = TestRequest::get()
            .insert_header(("HX-Request", "true"))
            .insert_header(("HX-Request-Type", "full"))
            .to_http_request();
        assert!(!is_partial_request(&req));
    }

    #[test]
    fn htmx_history_restores_need_the_full_page() {
        let req = TestRequest::get()
            .insert_header(("HX-Request", "true"))
            .insert_header(("HX-History-Restore-Request", "true"))
            .to_http_request();
        assert!(!is_partial_request(&req));
    }

    #[test]
    fn content_is_wrapped_into_the_index_template() {
        let page = wrap_content_into_full_page("Test garden", "<p id=\"marker\">hi</p>");

        assert!(page.starts_with("<!doctype html>"));
        assert!(page.contains("<title>Test garden</title>"));
        assert!(page.contains("<p id=\"marker\">hi</p>"));
        assert!(!page.contains("{{APPNAME}}") && !page.contains("{{CONTENT}}"));
    }
}
