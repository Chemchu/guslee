use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Article {
    id: u8,
    language: String,
    title: String,
    content: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
}

impl Article {
    pub fn get_content(&self) -> &String {
        &self.content
    }
}
