use actix_web::{get, http::header::AcceptLanguage, web, HttpRequest, HttpResponse, Responder};
use askama_actix::Template;
use compliments::Compliment;

use crate::{
    http_service::ResponseData,
    i18n::{self, to_language},
    md_service::{get_not_found_markdown, render_markdown, Article},
    AppState,
};

pub mod compliments;

#[derive(Template)]
#[template(path = "landing_page.html")]
pub struct LandingPage {
    translator: i18n::translator::Translator,
    compliment: Compliment,
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
        translator: i18n::translator::Translator::new(),
        compliment: Compliment::new(0),
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}

#[get("/articles")]
pub async fn articles_page() -> impl Responder {
    let template = ArticlesPage {
        translator: i18n::translator::Translator::new(),
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}

#[get("/articles/{article_id}")]
pub async fn article_page(
    req: HttpRequest,
    data: web::Data<AppState>,
    accept_language: web::Header<AcceptLanguage>,
) -> impl Responder {
    let article_id = req.match_info().get("article_id").unwrap_or("0");
    let languages = accept_language.ranked();
    let language = to_language(&languages);

    let article: Option<ResponseData<Article>> = data
        .http_service
        .get(
            &"blogs".to_string(),
            &format!("id=eq.{}&language=eq.{}&select=*", &article_id, &language),
        )
        .await;

    let md: String = if let Some(existing_article) = article {
        existing_article.content.get_content().to_owned()
    } else {
        get_not_found_markdown()
    };

    let html = render_markdown(&md);

    let template = ArticlePage {
        translator: i18n::translator::Translator::new(),
        content: html,
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}
