use actix_web::{
    Responder, get,
    web::{self, Html},
};
use maud::html;
use serde::{Deserialize, Serialize};

use crate::routes::AppState;

#[derive(Serialize, Deserialize)]
struct News {
    title: String,
    file_path: String,
    date: String,
    description: String,
    tags: Vec<String>,
}

#[get("/news")]
pub async fn news_page(app_state: web::Data<AppState>) -> impl Responder {
    let query = "SELECT 
        file_path,
        metadata.date AS date,
        metadata.description AS description,
        metadata.title AS title,
        metadata.tags AS tags
    FROM posts
    ORDER BY date DESC";
    let news = app_state.search_engine.raw_query::<Vec<News>>(query).await;

    // TODO: remove prose when returning
    let h = html! {
        h1 {
            "Recent News (Work in progress)"
        }
        ol {
            @for n in news {
                li {
                    (n.title)
                }
            }
        }
    };

    Html::new(h)
}
