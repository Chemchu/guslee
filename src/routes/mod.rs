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

#[derive(serde::Deserialize)]
struct Pagination {
    offset: Option<u32>,
    limit: Option<u32>,
}

#[derive(Template)]
#[template(path = "landing_page.html")]
pub struct LandingPage {
    translator: i18n::translator::Translator,
    scripts: Vec<String>,
}

#[derive(Template)]
#[template(path = "articles_page.html")]
pub struct ArticlesPage {
    scripts: Vec<String>,
}

#[derive(Template)]
#[template(path = "articles_list.html")]
pub struct ArticlesList {
    articles: Vec<Article>,
}

#[derive(Template)]
#[template(path = "article.html")]
pub struct ArticlePage {
    title: String,
    contents: Vec<String>, // Separado por parrafos
    date: String,
}

#[get("/")]
pub async fn landing_page() -> impl Responder {
    let template = LandingPage {
        translator: i18n::translator::Translator::new(),
        scripts: vec!["/_static/scroll.js".to_string()],
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}

#[get("/articles")]
pub async fn articles_page() -> impl Responder {
    let template = ArticlesPage {
        scripts: vec!["/_static/articles.js".to_string()],
    };

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}

#[get("/articles/content")]
pub async fn articles_content(
    data: web::Data<AppState>,
    accept_language: web::Header<AcceptLanguage>,
    pagination: web::Query<Pagination>,
) -> impl Responder {
    let languages = accept_language.ranked();
    let language = to_language(&languages);
    let offset = pagination.offset.unwrap_or(0);
    let limit = pagination.limit.unwrap_or(4);

    let articles: Option<ResponseData<Vec<Article>>> = data
        .http_service
        .get(
            &"blogs".to_string(),
            &format!(
                "language=eq.{}&select=*&order=created_at.desc&offset={}&limit={}",
                &language, offset, limit
            ),
        )
        .await;

    let template = ArticlesList {
        articles: if let Some(a) = articles {
            a.content
        } else {
            vec![]
        },
    };

    if template.articles.is_empty() {
        return HttpResponse::NotFound().body("No articles found");
    }

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
            title: ar.title.clone(),
            contents: ar.content.lines().map(|line| line.to_string()).collect(),
            date: ar.get_date(),
        };

        let reply_html = askama::Template::render(&template).unwrap();

        HttpResponse::Ok().body(reply_html)
    } else {
        let template = ArticlePage {
            title: "Not Found!".to_string(),
            contents: vec![get_not_found_markdown()],
            date: OffsetDateTime::now_utc().to_string(),
        };

        let reply_html = askama::Template::render(&template).unwrap();

        HttpResponse::Ok().body(reply_html)
    }
}
