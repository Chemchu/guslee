use actix_web::{
    Responder, get,
    web::{self, Html},
};
use markdown::{Constructs, Options, ParseOptions};
use search_engine::SearchEngine;
use std::sync::OnceLock;

pub mod chess_routes;
pub mod graph_routes;
pub mod news_routes;
pub mod posts_routes;

static INDEX_TEMPLATE: OnceLock<String> = OnceLock::new();

pub struct AppState {
    pub app_name: String,
    pub garden_path: String,
    pub search_engine: std::sync::Arc<SearchEngine>,
}

#[get("/v2")]
pub async fn landing_v2() -> impl Responder {
    load_html_page("news")
}

#[get("/")]
pub async fn landing(app_state: web::Data<AppState>) -> impl Responder {
    let welcome_path = format!("{}/welcome.md", app_state.garden_path);

    let content = match std::fs::read_to_string(&welcome_path) {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read welcome.md: {}", e);
            return Html::new(wrap_markdown_with_whole_page(
                &app_state.app_name,
                "<p>Error loading welcome page</p>",
            ));
        }
    };

    let frontmatter = Options {
        parse: ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    Html::new(wrap_markdown_with_whole_page(
        &app_state.app_name,
        &markdown::to_html_with_options(&content, &frontmatter).unwrap(),
    ))
}

fn wrap_markdown_with_whole_page(app_name: &str, content: &str) -> String {
    let html = INDEX_TEMPLATE.get_or_init(|| {
        let template_path =
            std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
        std::fs::read_to_string(format!("{}/index.html", template_path))
            .expect("Failed to read index.html template")
    });

    html.replace("{{APPNAME}}", app_name)
        .replace("{{CONTENT}}", content)
}

fn load_html_page(html_file: &str) -> Html {
    let template_path =
        std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
    let page = std::fs::read_to_string(format!("{}/{}.html", template_path, html_file))
        .expect("Failed to read chess_page.html template");

    Html::new(&page)
}
