use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use askama_actix::Template;

use crate::{http_service::ResponseData, i18n, md_service::Article, AppState};

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


    let article: ResponseData<Article> = data
        .http_service
        .get(
            &"blogs".to_string(),
            &format!("id=eq.{}&language=eq.{}&select=*", &article_id, &language),
        )
        .await;

    let article_md = article.content.get_content();

    let mut html_output = String::new();
    let parser = pulldown_cmark::Parser::new(article_md.as_str());
    pulldown_cmark::html::push_html(&mut html_output, parser);

    let template = ArticlePage {
        // TODO: Add State management to avoid creating a new Translator instance every time
        translator: i18n::translator::Translator::new(),
        content: html_output,
    };

    let reply_html = askama::Template::render(&template).unwrap();

    // TODO: Fetch markdown article from database
    HttpResponse::Ok().body(reply_html)
}
