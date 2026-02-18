use actix_web::{
    get,
    web::{self, Html},
};
use maud::html;

use crate::controllers::AppState;

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(render_metadata);
}

#[get("/metadata/{post:.*}")]
async fn render_metadata(app_state: web::Data<AppState>, path: web::Path<String>) -> Html {
    let file_path = format!("{}.md", path.as_str());
    let post = app_state.post_search_engine.get_post(&file_path).await;

    match post {
        Some(p) => {
            let html = html! {
                div
                class="flex flex-col gap-4"
                {
                    h4
                    class="text-2xl"
                    {
                        "Metadata"
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
