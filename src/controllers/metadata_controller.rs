use crate::controllers::{AppState, tags_controller::tag_url};
use actix_web::{
    get,
    web::{self, Html},
};
use maud::html;
pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(render_metadata);
}
#[get("/metadata/{post:.*}")]
async fn render_metadata(app_state: web::Data<AppState>, path: web::Path<String>) -> Html {
    let file_path = format!("{}.md", path.as_str());
    let post = app_state.post_search_engine.get_post(&file_path).await;
    match post {
        Some(p) => {
            let backlinks = app_state.post_search_engine.get_backlinks(&file_path).await;
            let html = html! {
                div class="flex flex-col gap-2 w-full" {
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
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Reading time" }
                            p class="text-sm font-semibold" { (p.metadata.reading_time) " min" }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Date" }
                            p class="text-sm font-semibold" { (p.metadata.date) }
                        }
                        @if let Some(last_updated) = &p.metadata.last_updated {
                            div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest" { "Last updated" }
                                time id="last-updated" datetime=(last_updated) class="text-sm font-semibold" { (last_updated) }
                            }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Tags" }
                            div class="flex flex-wrap gap-1" {
                                @for tag in &p.metadata.tags {
                                    a
                                    href=(tag_url(tag))
                                    class="px-3 py-1 text-xs font-medium bg-primary-color/10 text-primary-color hover:bg-primary-color/25 rounded-full transition-colors"
                                    { (tag) }
                                }
                            }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { "Linked mentions" }
                            @if backlinks.is_empty() {
                                p class="text-zinc-500 text-sm" { "No notes link here yet." }
                            } @else {
                                ul id="backlinks" class="flex flex-col gap-1" {
                                    @for backlink in &backlinks {
                                        li {
                                            a href=(format!(
                                                "/posts/{}",
                                                backlink
                                                    .file_path
                                                    .strip_suffix(".md")
                                                    .unwrap_or(&backlink.file_path)
                                            ))
                                            class="block text-zinc-100 hover:text-primary-color text-sm truncate"
                                            title=(backlink.title)
                                            { (backlink.title) }
                                        }
                                    }
                                }
                            }
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
                p class="text-zinc-500 text-sm" { "No metadata found for this post." }
            };
            Html::new(html)
        }
    }
}
