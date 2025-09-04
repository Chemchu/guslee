use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Html},
};
use std::{fs, io};
use tantivy::IndexReader;

pub struct AppState {
    pub app_name: String,
    pub posts_reader: IndexReader,
}

#[get("/")]
pub async fn landing() -> impl Responder {
    let content: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/garden/welcome.md"));

    Html::new(wrap_markdown_with_whole_page(&markdown::to_html(content)))
}

#[get("/{post:.*}")]
pub async fn post(req: HttpRequest, route: web::Path<String>) -> impl Responder {
    let content: io::Result<String> = fs::read_to_string(format!("./garden/{}.md", route));

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(match content {
            Ok(md) => markdown::to_html(&md),
            Err(_err) => String::from("Fallback page"),
        })
    } else {
        Html::new(match content {
            Ok(md) => wrap_markdown_with_whole_page(&markdown::to_html(&md)),
            Err(_err) => String::from("Fallback page"),
        })
    }
}

#[get("/search/{post:.*}")]
pub async fn search_post(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    route: web::Path<String>,
) -> impl Responder {
    /* if route.is_empty() {
        "Hello"
    } */
    "World"
}

fn wrap_markdown_with_whole_page(content: &str) -> String {
    let html: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index.html"));

    html.replace("{{CONTENT}}", content)
}
