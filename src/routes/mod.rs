use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use askama_actix::Template;

use crate::{i18n, AppState};

#[derive(Template)]
#[template(path = "landing_page.html")]
pub struct LandingPage {
    translator: i18n::translator::Translator,
}

#[derive(Template)]
#[template(path = "articles_page.html")]
pub struct ArticlesPage {
    translator: i18n::translator::Translator,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct ArticlePage {
    translator: i18n::translator::Translator,
    content: String,
}

#[get("/")]
pub async fn landing_page() -> impl Responder {
    let template = LandingPage {
        // TODO: Add State management to avoid creating a new Translator instance every time
        translator: i18n::translator::Translator::new(),
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}

#[get("/articles")]
pub async fn articles_page() -> impl Responder {
    let template = ArticlesPage {
        // TODO: Add State management to avoid creating a new Translator instance every time
        translator: i18n::translator::Translator::new(),
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}

#[get("/articles/{article_id}")]
pub async fn article_page(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let article_id = req.match_info().get("article_id").unwrap_or("0");
    let language = "en";

    let article = reqwest::get(format!(
        "{}/rest/v1/blogs?id=eq.{}&language={}&select=*",
        &data.supabase_url, article_id, language
    ))
    .await;

    if article.is_err() {
        panic!("{}", article.err().unwrap())
    }

    // TODO: Transform this into a struct similar to the supabase table
    let _article_md = article.unwrap().text().await.unwrap();

    let mut html_output = String::new();
    let parser = pulldown_cmark::Parser::new(_article_md.as_str());
    pulldown_cmark::html::push_html(&mut html_output, parser);

    let template = ArticlePage {
        // TODO: Add State management to avoid creating a new Translator instance every time
        translator: i18n::translator::Translator::new(),
        content: html_output,
    };

    let reply_html = askama::Template::render(&template).unwrap();

    println!("{}", reply_html);

    // TODO: Fetch markdown article from database
    HttpResponse::Ok().body(reply_html)
}
