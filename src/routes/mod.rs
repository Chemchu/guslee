use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Html},
};
use maud::html;
use search_engine::{SearchEngine, SearchResult};
use serde::Deserialize;
use std::{fs, io};

pub struct AppState {
    pub app_name: String,
    pub search_engine: std::sync::Arc<SearchEngine>,
}

#[derive(Deserialize)]
struct Params {
    limit: Option<usize>,
}

const DEFAULT_SEARCH_LIMIT: usize = 100;

#[get("/")]
pub async fn landing(app_state: web::Data<AppState>) -> impl Responder {
    let content: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/garden/welcome.md"));

    Html::new(wrap_markdown_with_whole_page(
        &app_state.app_name,
        &markdown::to_html(content),
    ))
}

#[get("/{post:.*}")]
pub async fn post(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    route: web::Path<String>,
) -> impl Responder {
    let content: io::Result<String> = fs::read_to_string(format!("./garden/{}.md", route));

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(match content {
            Ok(md) => markdown::to_html(&md),
            Err(_err) => String::from("Fallback page"),
        })
    } else {
        Html::new(match content {
            Ok(md) => wrap_markdown_with_whole_page(&app_state.app_name, &markdown::to_html(&md)),
            Err(_err) => String::from("Fallback page"),
        })
    }
}

#[get("/search/{query}")]
pub async fn search_post(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    route: web::Path<String>,
    params: web::Query<Params>,
) -> impl Responder {
    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        if route.is_empty() {
            return Html::new(String::from("Fallback page"));
        }
        let result: SearchResult = app_state
            .search_engine
            .exec_query(route.as_str(), params.limit.unwrap_or(DEFAULT_SEARCH_LIMIT));

        let html = html! {
            ul {
                @for matching_file in result.matching_files {
                    li {
                        a href=(format!(
                            "/{}",
                            matching_file
                                .file_path()
                                .strip_suffix(".md")
                                .unwrap_or(matching_file.file_path())
                        ))
                        hx-target="#content-section"
                        hx-swap="innerHTML"
                        {
                            (matching_file
                                .file_name()
                                .strip_suffix(".md")
                                .unwrap_or(matching_file.file_path()))
                        }
                    }
                }
            }
        };

        Html::new(html)
    } else {
        Html::new(String::from("Only HTMX requests for search engine"))
    }
}

fn wrap_markdown_with_whole_page(app_name: &str, content: &str) -> String {
    let html: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index.html"));

    html.replace("{{APPNAME}}", app_name)
        .replace("{{CONTENT}}", content)
}
