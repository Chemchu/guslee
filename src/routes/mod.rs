use actix_web::{get, HttpResponse, Responder};
use askama_actix::Template;

use crate::i18n;

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
