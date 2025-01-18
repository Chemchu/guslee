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

pub fn get_not_found_markdown() -> String {
    "# 🚧 **404 Not Found** 🚧

---

## Uh-oh! Looks like you're lost... 🗺️

The page you’re looking for doesn’t exist or has been moved.  
Don’t worry, we’ll help you find your way!

---

### What you can do:

- [🏠 Go Back to Home](/)  
- [🔍 Search for what you need](#)  
- [📧 Contact Support](mailto:support@example.com)  

---

![Lost in Space](https://via.placeholder.com/600x300?text=Lost+in+Space)

> *'Not all those who wander are lost, but this page certainly is!'*  
> — *J.R.R. Tolkien* (probably)

---
"
    .to_string()
}
