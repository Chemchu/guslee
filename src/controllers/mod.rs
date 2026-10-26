use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web::Html};
use chess_module::LichessState;
use games_module::SteamState;
use music_module::SpotifyState;
use search_engine::PostsSearchEngine;
use std::sync::{Arc, OnceLock};

pub mod chess_controller;
pub mod fallback_controller;
pub mod feed_controller;
pub mod graph_controller;
pub mod metadata_controller;
pub mod music_controller;
pub mod news_controller;
pub mod obsidian_markdown;
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

pub const SITE_DESCRIPTION: &str = "My personal digital garden, built with Rust + HTMX";

/// What a page tells browsers, feed readers and link previews about itself.
#[derive(Default)]
pub struct PageMeta {
    /// Shown before the site name. `None` for pages that are the site itself.
    pub title: Option<String>,
    /// Defaults to [`SITE_DESCRIPTION`].
    pub description: Option<String>,
    /// Posts are articles, everything else is a website.
    pub is_article: bool,
}

impl PageMeta {
    pub fn titled(title: impl Into<String>) -> PageMeta {
        PageMeta {
            title: Some(title.into()),
            ..PageMeta::default()
        }
    }

    pub fn full_title(&self, app_name: &str) -> String {
        match &self.title {
            Some(title) => format!("{title} · {app_name}"),
            None => app_name.to_string(),
        }
    }

    /// `<head>` tags: title, description, Open Graph and the RSS feed.
    fn head(&self, app_name: &str, url: Option<&str>) -> String {
        let description = self.description.as_deref().unwrap_or(SITE_DESCRIPTION);
        let og_type = if self.is_article {
            "article"
        } else {
            "website"
        };

        maud::html! {
            title { (self.full_title(app_name)) }
            meta name="description" content=(description);
            meta property="og:site_name" content=(app_name);
            meta property="og:type" content=(og_type);
            meta property="og:title" content=(self.title.as_deref().unwrap_or(app_name));
            meta property="og:description" content=(description);
            @if let Some(url) = url {
                meta property="og:url" content=(url);
                link rel="canonical" href=(url);
            }
            meta name="twitter:card" content="summary";
            link rel="alternate" type="application/rss+xml" title=(app_name) href="/rss.xml";
        }
        .into_string()
    }
}

/// Scheme and host the request came in through, proxies included.
pub fn base_url(req: &HttpRequest) -> String {
    let info = req.connection_info();
    format!("{}://{}", info.scheme(), info.host())
}

/// Percent-encodes anything but unreserved characters, keeping `/` so the
/// result still reads as a path.
pub fn encode_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

pub fn wrap_content_into_full_page(app_name: &str, content: &str) -> String {
    wrap_content_with_meta(app_name, content, &PageMeta::default(), None)
}

pub fn wrap_content_with_meta(
    app_name: &str,
    content: &str,
    meta: &PageMeta,
    url: Option<&str>,
) -> String {
    let html = INDEX_TEMPLATE.get_or_init(|| {
        let template_path =
            std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
        std::fs::read_to_string(format!("{}/index.html", template_path))
            .expect("Failed to read index.html template")
    });

    html.replace("{{HEAD}}", &meta.head(app_name, url))
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
    render_page_with_meta(app_state, req, content, &PageMeta::default())
}

/// Like [`render_page`], with the page's own title and description. Partial
/// responses carry a `<title>` too, which htmx moves into the browser tab.
pub fn render_page_with_meta(
    app_state: &AppState,
    req: &HttpRequest,
    content: &str,
    meta: &PageMeta,
) -> Html {
    if is_partial_request(req) {
        let title = maud::html! { title { (meta.full_title(&app_state.app_name)) } };
        Html::new(format!("{}{content}", title.into_string()))
    } else {
        let url = format!("{}{}", base_url(req), req.path());
        Html::new(wrap_content_with_meta(
            &app_state.app_name,
            content,
            meta,
            Some(&url),
        ))
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
        assert!(!page.contains("{{HEAD}}") && !page.contains("{{CONTENT}}"));
        assert!(page.contains(&format!(
            "<meta name=\"description\" content=\"{SITE_DESCRIPTION}\">"
        )));
        assert!(page.contains(r#"<meta property="og:type" content="website">"#));
        assert!(!page.contains("og:url"), "no URL without a request");
    }

    #[test]
    fn page_meta_fills_titles_descriptions_and_links() {
        let meta = PageMeta {
            title: Some("Fog <notes>".to_string()),
            description: Some("About \"fog\"".to_string()),
            is_article: true,
        };

        let page = wrap_content_with_meta("Garden", "", &meta, Some("https://g.dev/posts/fog"));

        assert!(page.contains("<title>Fog &lt;notes&gt; · Garden</title>"));
        assert!(page.contains(r#"<meta property="og:title" content="Fog &lt;notes&gt;">"#));
        assert!(page.contains(r#"<meta name="description" content="About &quot;fog&quot;">"#));
        assert!(page.contains(r#"<meta property="og:type" content="article">"#));
        assert!(page.contains(r#"<meta property="og:url" content="https://g.dev/posts/fog">"#));
        assert!(page.contains(r#"<link rel="alternate" type="application/rss+xml""#));
    }

    #[test]
    fn paths_are_percent_encoded_except_slashes() {
        assert_eq!(encode_path("gaming/horror"), "gaming/horror");
        assert_eq!(encode_path("digital garden"), "digital%20garden");
        assert_eq!(encode_path("c#ñ"), "c%23%C3%B1");
    }
}
