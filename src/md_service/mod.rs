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

pub fn render_markdown(md: &str) -> String {
    let mut html_output = String::new();
    let parser = pulldown_cmark::Parser::new(md);
    pulldown_cmark::html::push_html(&mut html_output, parser);

    html_output
}
