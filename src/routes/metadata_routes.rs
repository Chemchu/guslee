use actix_web::{
    get,
    web::{self, Html},
};
use maud::html;

use crate::routes::AppState;

#[get("/metadata/{post}")]
pub async fn render_metadata(app_state: web::Data<AppState>, path: web::Path<String>) -> Html {
    let file_path = format!("{}.md", path.as_str());
    let post = app_state.search_engine.get_post(&file_path).await;

    match post {
        Some(p) => {
            let html = html! {
                div
                class="flex flex-col gap-4"
                {
                    h4
                    class="text-2xl"
                    {
                        "About this post"
                    }
                    div
                    class="flex flex-col text-md"
                    {
                        details open="true" {
                            summary class="text-lg" {
                                "Title"
                            }
                            (p.metadata.title)
                        }
                        details open="true" {
                            summary class="text-lg" {
                                "Description"
                            }
                            (p.metadata.description)
                        }
                        details open="true" {
                            summary class="text-lg" {
                                "Date"
                            }
                            (p.metadata.date)
                        }
                    }
                }
            };

            Html::new(html)
        }
        None => {
            let html = html! {
                div
                class="flex flex-col text-lg"
                {
                    span
                    class="text-2xl"
                    {
                        "Metadata not found"
                    }
                }
            };

            Html::new(html)
        }
    }
}
