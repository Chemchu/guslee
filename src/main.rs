use actix_web::{
    App, HttpServer, Responder, get,
    web::{self, Html},
};
use std::{fs, io};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(route))
        .bind(("127.0.0.1", 3000))?
        .run()
        .await
}

#[get("/{route}")]
async fn route(route: web::Path<String>) -> impl Responder {
    let content: io::Result<String> =
        fs::read_to_string(format!("./garden/{}.md", route.to_lowercase()));

    Html::new(match content {
        Ok(md) => markdown::to_html(&md),
        Err(_err) => String::from("Fallback page"),
    })
}
