use actix_web::{
    Responder, get,
    web::{self, Html},
};
use maud::PreEscaped;
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
    let news: Vec<News> = app_state
        .search_engine
        .raw_query::<Vec<News>>(query)
        .await
        .iter()
        .map(|n| {
            let path = format!("/{}", n.file_path.replace(".md", ""));
            News {
                title: n.title.clone(),
                file_path: path,
                date: n.date.clone(),
                description: n.description.clone(),
                tags: n.tags.clone(),
            }
        })
        .clone()
        .collect();

    let h = html! {
        div
        class="flex flex-col gap-4 justify-between"
        {
            h1 {
                "Latest stuff"
            }
            ol
            class="flex flex-col gap-4"
            {
                @for n in news {
                    li
                    class="flex flex-col gap-1"
                    {
                        a
                        class="text-xl cursor-pointer hover:text-primary-color"
                        href=(n.file_path)
                        hx-target="#content-section"
                        hx-trigger="click"
                        hx-swap="innerHTML transition:true"
                        hx-on::after-request="document.getElementById('content-section').classList.add('prose')"
                        {
                            div
                            class="flex gap-2"
                            {
                                span class="text-primary-color"
                                {
                                    (PreEscaped(r#"&#8226;"#))
                                }
                                (n.title)
                            }
                        }
                        div
                        class="md:flex md:gap-4 justify-between"
                        {
                            span
                            class="text-md"
                            {
                                (n.description)
                            }
                            span
                            class="text-md hidden md:flex"
                            {
                                (n.date)
                            }
                        }
                        span
                        class="flex md:hidden text-md"
                        {
                            (n.date)
                        }
                    }
                }
            }
        }
    };

    Html::new(h)
}
