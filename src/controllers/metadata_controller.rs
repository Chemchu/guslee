use crate::controllers::{AppState, tags_controller::tag_url};
use crate::i18n::Locale;
use actix_web::{
    get,
    web::{self, Html},
};
use maud::html;
pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(render_metadata);
}
#[get("/metadata/{post:.*}")]
async fn render_metadata(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    locale: Locale,
) -> Html {
    let t = locale.texts();
    let file_path = format!("{}.md", path.as_str());
    let post = app_state
        .post_search_engine
        .get_post(&file_path, locale.as_str())
        .await;
    match post {
        Some(p) => {
            let backlinks = app_state.post_search_engine.get_backlinks(&file_path).await;
            let mentions = app_state
                .post_search_engine
                .get_unlinked_mentions(&file_path)
                .await;
            let html = html! {
                div class="flex flex-col gap-2 w-full" {
                    div class="flex flex-col gap-px border-t border-shade-color" {
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_title) }
                            p class="text-zinc-100 text-sm font-semibold" { (p.metadata.title) }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_description) }
                            p class="text-zinc-100 text-sm" { (p.metadata.description) }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_reading_time) }
                            p class="text-sm font-semibold" { (p.metadata.reading_time) " " (t.meta_minutes) }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_date) }
                            p class="text-sm font-semibold" { (p.metadata.date) }
                        }
                        @if let Some(last_updated) = &p.metadata.last_updated {
                            div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_last_updated) }
                                time id="last-updated" datetime=(last_updated) class="text-sm font-semibold" { (last_updated) }
                            }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_tags) }
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
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_linked_mentions) }
                            @if backlinks.is_empty() {
                                p class="text-zinc-500 text-sm" { (t.meta_no_backlinks) }
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
                        @if !mentions.is_empty() {
                            div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                                p
                                class="text-zinc-500 text-xs uppercase tracking-widest"
                                title=(t.meta_unlinked_hint)
                                { (t.meta_unlinked_mentions) }
                                ul id="unlinked-mentions" class="flex flex-col gap-2" {
                                    @for mention in &mentions {
                                        li class="flex flex-col" {
                                            a href=(format!(
                                                "/posts/{}",
                                                mention
                                                    .file_path
                                                    .strip_suffix(".md")
                                                    .unwrap_or(&mention.file_path)
                                            ))
                                            class="block text-zinc-100 hover:text-primary-color text-sm truncate"
                                            title=(mention.title)
                                            { (mention.title) }
                                            p class="text-zinc-500 text-xs line-clamp-2" {
                                                (mention.before)
                                                mark class="bg-primary-color/25 text-zinc-200 rounded-sm px-0.5" { (mention.matched) }
                                                (mention.after)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        div class="flex flex-col gap-1 px-4 py-3 border-b border-r border-shade-color border-l hover:border-l-2 border-l-shade-color hover:border-l-primary-color hover:bg-shade-color transition-colors" {
                            p class="text-zinc-500 text-xs uppercase tracking-widest" { (t.meta_source) }
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
                p class="text-zinc-500 text-sm" { (t.meta_not_found) }
            };
            Html::new(html)
        }
    }
}
