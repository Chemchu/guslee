use actix_web::{get, HttpResponse, Responder};
use askama::Template;

#[derive(Template)]
#[template(path = "landing_page.html")]
pub struct LandingPage {}

#[get("/")]
pub async fn landing_page() -> impl Responder {
    let template = LandingPage {};

    let reply_html = askama::Template::render(&template).unwrap();

    HttpResponse::Ok().body(reply_html)
}
