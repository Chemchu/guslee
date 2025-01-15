use actix_web::cookie::time::OffsetDateTime;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
pub struct Article {
    id: u8,
    language: String,
    title: String,
    content: String,
    created_at: OffsetDateTime,
}

impl Article {
    pub fn get_content(&self) -> &String {
        &self.content
    }
}
