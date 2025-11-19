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
                class="flex flex-col text-lg"
                {
                    span
                    class="text-2xl"
                    {
                        "Metadata"
                    }
                    span {
                        "Title: " (p.metadata.title)
                    }
                    span {
                        "Description: " (p.metadata.description)
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
