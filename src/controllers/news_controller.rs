use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Html},
};
use maud::PreEscaped;
use maud::html;
use serde::{Deserialize, Serialize};

use crate::controllers::{AppState, wrap_content_into_full_page};

#[derive(Serialize, Deserialize)]
struct News {
    title: String,
    file_path: String,
    date: String,
    description: String,
    tags: Vec<String>,
}

#[get("/news")]
pub async fn news_page(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
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
        class="flex flex-col gap-4 justify-between p-4 md:p-6 lg:p-8 overflow-auto w-full"
        {
            h1
            class="text-4xl pb-4"
            {
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
                        href=(format!("/posts/{}", n.file_path))
                        hx-target="#content-section"
                        hx-trigger="click"
                        hx-swap="innerHTML transition:true"
                        {
                            div
                            class="flex gap-2"
                            {
                                span class="text-primary-color"
                                {
                                    (PreEscaped(r#"&#8226;"#))
                                }
                                div
                                class="md:flex md:gap-4 w-full justify-between"
                                {
                                    div
                                    class="flex flex-col w-full"
                                    {
                                        span
                                        {
                                            (n.title)
                                        }
                                        span
                                        class="text-base"
                                        {
                                            (n.description)
                                        }
                                    }
                                    span
                                    class="text-base hidden md:flex"
                                    {
                                        (n.date)
                                    }
                                }
                                span
                                class="flex md:hidden text-base"
                                {
                                    (n.date)
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(h)
    } else {
        Html::new(wrap_content_into_full_page(&app_state.app_name, &h.0))
    }
}
