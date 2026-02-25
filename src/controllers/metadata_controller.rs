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
                div class="flex flex-col gap-2 w-full" {
                    div class="flex items-center gap-3 w-full" {
                        p class="text-primary-color text-sm font-semibold uppercase tracking-wider" { "Metadata" }
                        div class="flex-1 h-px bg-shade-color" {}
                    }
                    div class="flex flex-col gap-px border-t border-shade-color" {
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Title" }
                            p class="text-zinc-100 text-sm font-semibold" { (p.metadata.title) }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Description" }
                            p class="text-zinc-100 text-sm" { (p.metadata.description) }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Date" }
                            p class="text-primary-color text-sm font-semibold" { (p.metadata.date) }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Post Source" }
                            a href={(p.metadata.post_source_url) "?plain=1"} target="_blank" rel="noopener noreferrer"
                            class="text-zinc-100 hover:text-primary-color text-sm truncate" { (p.metadata.post_source_url) }
                        }
                    }
                }
            };
            Html::new(html)
        }
        None => {
            let html = html! {
                div class="flex flex-col gap-3 p-6 border border-shade-color" {
                    div class="flex items-center gap-3" {
                        p class="text-primary-color text-sm font-semibold uppercase tracking-wider" { "Metadata" }
                        div class="flex-1 h-px bg-shade-color" {}
                    }
                    p class="text-zinc-500 text-sm" { "No metadata found for this post." }
                }
            };
            Html::new(html)
        }
    }
}
