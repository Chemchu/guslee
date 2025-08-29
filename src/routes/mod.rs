use actix_web::{
    Responder, get,
    web::{self, Html},
};
use std::{fs, io};

#[get("/")]
pub async fn landing() -> impl Responder {
    let path = "./templates/index.html";
    let content: String =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("File not found: {}", path));

    Html::new(content)
}

#[get("/{post}")]
pub async fn post(route: web::Path<String>) -> impl Responder {
    let content: io::Result<String> = fs::read_to_string(format!("./garden/{}.md", route));

    Html::new(match content {
        Ok(md) => markdown::to_html(&md),
        Err(_err) => String::from("Fallback page"),
    })
}
