use actix_web::{get, http::header::AcceptLanguage, web, HttpRequest, HttpResponse, Responder};
use askama_actix::Template;
use time::OffsetDateTime;

use crate::{
    http_service::ResponseData,
    i18n::{self, to_language},
    md_service::{get_not_found_markdown, Article},
    AppState,
};

pub mod compliments;

#[derive(Template)]
#[template(path = "landing_page.html")]
pub struct LandingPage {
    translator: i18n::translator::Translator,
}

#[derive(Template)]
#[template(path = "articles_page.html")]
pub struct ArticlesPage {
    translator: i18n::translator::Translator,
    articles: Vec<Article>,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct ArticlePage {
    translator: i18n::translator::Translator,
    title: String,
    date: OffsetDateTime,
    contents: Vec<String>, // Separado por parrafos
}

#[get("/")]
pub async fn landing_page() -> impl Responder {
    let template = LandingPage {
        translator: i18n::translator::Translator::new(),
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}

#[get("/articles")]
pub async fn articles_page(
    data: web::Data<AppState>,
    accept_language: web::Header<AcceptLanguage>,
) -> impl Responder {
    let languages = accept_language.ranked();
    let language = to_language(&languages);

    let articles: Option<ResponseData<Vec<Article>>> = data
        .http_service
        .get(
            &"blogs".to_string(),
            &format!("language=eq.{}&select=*&order=created_at.desc", &language),
        )
        .await;

    let template = ArticlesPage {
        translator: i18n::translator::Translator::new(),
        articles: if let Some(a) = articles {
            a.content
        } else {
            vec![]
        },
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

    let article: Option<ResponseData<Vec<Article>>> = data
        .http_service
        .get(
            &"blogs".to_string(),
            &format!("id=eq.{}&language=eq.{}&select=*", &article_id, &language),
        )
        .await;

    let article = if let Some(existing_article) = article {
        existing_article.content.first().cloned()
    } else {
        None
    };

    if let Some(ar) = article {
        let template = ArticlePage {
            translator: i18n::translator::Translator::new(),
            title: ar.title.clone(),
            date: ar.created_at,
            contents: ar.content.lines().map(|line| line.to_string()).collect(),
        };

        let reply_html = askama::Template::render(&template).unwrap();

        HttpResponse::Ok().body(reply_html)
    } else {

        let template = ArticlePage {
            translator: i18n::translator::Translator::new(),
            title: "Not Found!".to_string(),
            date: OffsetDateTime::now_utc(),
            contents: vec![get_not_found_markdown()],
        };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
    }

}
