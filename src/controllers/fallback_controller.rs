use actix_web::{Responder, get};

#[get("/{url:.*}")]
pub async fn fallback_route() -> impl Responder {
    String::from("Fallback page")
}
