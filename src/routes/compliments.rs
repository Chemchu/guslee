use actix_web::{get, web, Responder};
use askama_actix::Template;

#[derive(Clone, Template)]
#[template(path = "compliment.html")]
pub struct Compliment {
    pub next_compliment_id: usize,
    pub compliment: &'static str,
}

impl Compliment {
    pub fn new(id: usize) -> Self {
        let c: Vec<&str> = vec![
            "the",
            "a beautiful",
            "an amazing",
            "way too handsome",
            "incredible smart",
            "funny",
        ];

        let index = if id >= c.len() { 0 } else { id };

        Self {
            next_compliment_id: index + 1,
            compliment: c[index],
        }
    }
}

#[get("/compliments/{compliment_id}")]
pub async fn compliments(path: web::Path<u32>) -> impl Responder {
    let compliment_id = path.into_inner() as usize;

    Compliment::new(compliment_id)
}
