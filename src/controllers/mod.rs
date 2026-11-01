use actix_web::{HttpRequest, HttpResponse, http::header::ContentType, web::Html};
use chess_module::LichessState;
use games_module::SteamState;
use music_module::SpotifyState;
use search_engine::{PostsSearchEngine, utils::DEFAULT_LANG};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::i18n::{LANG_PARAM, Locale, Texts};

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

static INDEX_TEMPLATES: OnceLock<HashMap<Locale, String>> = OnceLock::new();
const TEXT_PLACEHOLDER: &str = "{{t:";

pub struct AppState {
    pub app_name: String,
    pub garden_path: String,
    pub lichess_state: LichessState,
    pub spotify_state: Arc<tokio::sync::Mutex<SpotifyState>>,
    pub steam_state: SteamState,
    pub post_search_engine: Arc<PostsSearchEngine>,
}

#[derive(Default)]
pub struct PageMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    pub is_article: bool,
    pub lang: Option<String>,
    pub translations: Vec<String>,
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

    fn head(&self, app_name: &str, url: Option<&str>, texts: &Texts) -> String {
        let description = self
            .description
            .as_deref()
            .unwrap_or(texts.site_description);
        let og_type = if self.is_article {
            "article"
        } else {
            "website"
        };
        let lang_url = |url: &str, lang: &str| format!("{url}?{LANG_PARAM}={lang}");
        let page_url = url.map(|url| match (&self.lang, self.translations.len() > 1) {
            (Some(lang), true) => lang_url(url, lang),
            _ => url.to_string(),
        });
        let feed_url = match self.lang.as_deref() {
            Some(lang) if lang != DEFAULT_LANG => lang_url("/rss.xml", lang),
            _ => "/rss.xml".to_string(),
        };

        maud::html! {
            title { (self.full_title(app_name)) }
            meta name="description" content=(description);
            meta property="og:site_name" content=(app_name);
            meta property="og:type" content=(og_type);
            meta property="og:title" content=(self.title.as_deref().unwrap_or(app_name));
            meta property="og:description" content=(description);
            @if let Some(page_url) = &page_url {
                meta property="og:url" content=(page_url);
                link rel="canonical" href=(page_url);
            }
            @if let (Some(url), true) = (url, self.translations.len() > 1) {
                @for lang in &self.translations {
                    link rel="alternate" hreflang=(lang) href=(lang_url(url, lang));
                }
                link rel="alternate" hreflang="x-default" href=(url);
            }
            meta name="twitter:card" content="summary";
            link rel="alternate" type="application/rss+xml" title=(app_name) href=(feed_url);
        }
        .into_string()
    }
}

pub fn base_url(req: &HttpRequest) -> String {
    let info = req.connection_info();
    format!("{}://{}", info.scheme(), info.host())
}

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

pub fn wrap_content_into_full_page(app_name: &str, content: &str, locale: Locale) -> String {
    wrap_content_with_meta(app_name, content, &PageMeta::default(), None, locale)
}

pub fn wrap_content_with_meta(
    app_name: &str,
    content: &str,
    meta: &PageMeta,
    url: Option<&str>,
    locale: Locale,
) -> String {
    let templates = INDEX_TEMPLATES.get_or_init(|| {
        let template_path =
            std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
        let template = std::fs::read_to_string(format!("{}/index.html", template_path))
            .expect("Failed to read index.html template");
        Locale::ALL
            .into_iter()
            .map(|locale| (locale, localize_template(&template, locale)))
            .collect()
    });

    templates[&locale]
        .replace("{{HEAD}}", &meta.head(app_name, url, locale.texts()))
        .replace("{{CONTENT}}", content)
}

fn localize_template(template: &str, locale: Locale) -> String {
    let texts = locale.texts();
    let mut html = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find(TEXT_PLACEHOLDER) {
        html.push_str(&rest[..start]);
        let after = &rest[start + TEXT_PLACEHOLDER.len()..];
        let Some(end) = after.find("}}") else {
            rest = &rest[start..];
            break;
        };
        let key = &after[..end];
        match texts.get(key) {
            Some(text) => html.push_str(&maud::html! { (text) }.into_string()),
            None => {
                log::warn!("Missing {} text for template key {key}", locale.as_str());
                html.push_str(key);
            }
        }
        rest = &after[end + "}}".len()..];
    }
    html.push_str(rest);

    html.replace("{{LANG}}", locale.as_str())
        .replace("{{LANGUAGE_SWITCHER}}", &language_switcher(locale))
}

fn language_switcher(current: Locale) -> String {
    maud::html! {
        nav aria-label=(current.texts().language_label) class="flex gap-2" {
            @for locale in Locale::ALL {
                a
                href=(format!("?{LANG_PARAM}={}", locale.as_str()))
                hx-boost="false"
                hreflang=(locale.as_str())
                lang=(locale.as_str())
                aria-current=[(locale == current).then_some("true")]
                class="uppercase hover:text-primary-color aria-[current=true]:text-primary-color"
                { (locale.as_str()) }
            }
        }
    }
    .into_string()
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
            Locale::negotiate(req),
        ))
    }
}

pub fn render_not_found(app_state: &AppState, req: &HttpRequest) -> HttpResponse {
    let locale = Locale::negotiate(req);
    let t = locale.texts();
    let content = maud::html! {
        div class="flex flex-col w-full gap-4 p-6 lg:p-8" {
            h1 class="text-4xl text-primary-color" { "404" }
            p { (t.not_found_message) }
            a href="/posts/welcome" hx-target="#main-section" class="hover:text-primary-color underline" {
                (t.not_found_back)
            }
        }
    }
    .into_string();

    let body = if is_partial_request(req) {
        content
    } else {
        wrap_content_into_full_page(&app_state.app_name, &content, locale)
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
        let page =
            wrap_content_into_full_page("Test garden", "<p id=\"marker\">hi</p>", Locale::En);

        assert!(page.starts_with("<!doctype html>"));
        assert!(page.contains("<title>Test garden</title>"));
        assert!(page.contains("<p id=\"marker\">hi</p>"));
        assert!(!page.contains("{{HEAD}}") && !page.contains("{{CONTENT}}"));
        assert!(page.contains(&format!(
            "<meta name=\"description\" content=\"{}\">",
            Locale::En.texts().site_description
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
            ..PageMeta::default()
        };

        let page = wrap_content_with_meta(
            "Garden",
            "",
            &meta,
            Some("https://g.dev/posts/fog"),
            Locale::En,
        );

        assert!(page.contains("<title>Fog &lt;notes&gt; · Garden</title>"));
        assert!(page.contains(r#"<meta property="og:title" content="Fog &lt;notes&gt;">"#));
        assert!(page.contains(r#"<meta name="description" content="About &quot;fog&quot;">"#));
        assert!(page.contains(r#"<meta property="og:type" content="article">"#));
        assert!(page.contains(r#"<meta property="og:url" content="https://g.dev/posts/fog">"#));
        assert!(page.contains(r#"<link rel="alternate" type="application/rss+xml""#));
    }

    #[test]
    fn translated_pages_link_every_language_version() {
        let meta = PageMeta {
            lang: Some("es".to_string()),
            translations: vec!["en".to_string(), "es".to_string()],
            ..PageMeta::titled("Niebla")
        };

        let page = wrap_content_with_meta(
            "Garden",
            "",
            &meta,
            Some("https://g.dev/posts/fog"),
            Locale::En,
        );

        assert!(page.contains(r#"<link rel="canonical" href="https://g.dev/posts/fog?lang=es">"#));
        assert!(
            page.contains(r#"<meta property="og:url" content="https://g.dev/posts/fog?lang=es">"#)
        );
        assert!(page.contains(
            r#"<link rel="alternate" hreflang="en" href="https://g.dev/posts/fog?lang=en">"#
        ));
        assert!(page.contains(
            r#"<link rel="alternate" hreflang="es" href="https://g.dev/posts/fog?lang=es">"#
        ));
        assert!(page.contains(
            r#"<link rel="alternate" hreflang="x-default" href="https://g.dev/posts/fog">"#
        ));
        assert!(page.contains(r#"href="/rss.xml?lang=es""#));
    }

    #[test]
    fn untranslated_pages_keep_a_plain_canonical_and_no_hreflang() {
        let meta = PageMeta {
            lang: Some("en".to_string()),
            translations: vec!["en".to_string()],
            ..PageMeta::titled("Fog")
        };

        let page = wrap_content_with_meta(
            "Garden",
            "",
            &meta,
            Some("https://g.dev/posts/fog"),
            Locale::En,
        );

        assert!(page.contains(r#"<link rel="canonical" href="https://g.dev/posts/fog">"#));
        assert!(!page.contains(r#"rel="alternate" hreflang"#));
        assert!(page.contains(r#"href="/rss.xml""#));
    }

    #[test]
    fn every_template_text_exists_in_every_language() {
        let template = std::fs::read_to_string("templates/index.html").unwrap();

        for locale in Locale::ALL {
            let html = localize_template(&template, locale);
            assert!(!html.contains(TEXT_PLACEHOLDER), "{}", locale.as_str());
            assert!(!html.contains("{{LANG"), "{}", locale.as_str());
            assert!(html.contains(&format!(r#"<html lang="{}">"#, locale.as_str())));
        }
    }

    #[test]
    fn spanish_pages_use_spanish_texts() {
        let page = wrap_content_into_full_page("Garden", "", Locale::Es);

        assert!(page.contains(r#"<html lang="es">"#));
        assert!(page.contains("INICIO"));
        assert!(page.contains(r#"placeholder="Busca cualquier post...""#));
        assert!(page.contains(&format!(
            r#"<meta name="description" content="{}">"#,
            Locale::Es.texts().site_description
        )));
        assert!(!page.contains(">HOME<"));
    }

    #[test]
    fn template_texts_are_html_escaped() {
        let html = localize_template(
            r#"<p title="{{t:switcher_help}}">{{t:tag_back}}</p>"#,
            Locale::En,
        );

        assert_eq!(
            html,
            r#"<p title="↑↓ to move · Enter to open · Esc to close">← All tags</p>"#
        );
        assert_eq!(
            localize_template("{{t:not_a_key}} {{t:unclosed", Locale::En),
            "not_a_key {{t:unclosed"
        );
    }

    #[test]
    fn the_language_switcher_marks_the_current_language() {
        let switcher = language_switcher(Locale::Es);

        assert!(switcher.contains(r#"<nav aria-label="Idioma""#));
        assert!(
            switcher.contains(r#"href="?lang=en" hx-boost="false" hreflang="en" lang="en" class="#)
        );
        assert!(switcher.contains(
            r#"href="?lang=es" hx-boost="false" hreflang="es" lang="es" aria-current="true""#
        ));
    }

    #[test]
    fn paths_are_percent_encoded_except_slashes() {
        assert_eq!(encode_path("gaming/horror"), "gaming/horror");
        assert_eq!(encode_path("digital garden"), "digital%20garden");
        assert_eq!(encode_path("c#ñ"), "c%23%C3%B1");
    }
}
