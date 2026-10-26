use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use maud::html;

use crate::controllers::{
    AppState, PageMeta, encode_path, render_not_found, render_page_with_meta,
};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(tags_index).service(tag_page);
}

pub fn tag_url(tag: &str) -> String {
    format!("/tags/{}", encode_path(tag))
}

const TAG_INDENT: [&str; 5] = ["pl-0", "pl-4", "pl-8", "pl-12", "pl-16"];

#[get("/tags")]
async fn tags_index(app_state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    render_tags_index(&app_state, &req).await
}

async fn render_tags_index(app_state: &AppState, req: &HttpRequest) -> HttpResponse {
    let tags = app_state.post_search_engine.tag_counts().await;

    let template = html! {
        div
        class="p-4 md:p-6 lg:p-8 overflow-auto w-full"
        {
            div
            class="max-w-3xl mx-auto"
            {
                div
                class="mb-8"
                {
                    h1 class="text-5xl md:text-6xl font-bold mb-2" { "TAGS" }
                    p class="text-zinc-500" { (tags.len()) " tags" }
                }
                ul
                id="tag-index"
                class="flex flex-col border-t border-shade-color"
                {
                    @for (tag, count) in &tags {
                        @let depth = tag.matches('/').count();
                        li
                        class={
                            "flex items-baseline justify-between gap-4 py-2 border-b border-shade-color "
                            (TAG_INDENT[depth.min(TAG_INDENT.len() - 1)])
                        }
                        {
                            a
                            href=(tag_url(tag))
                            title={ "#" (tag) }
                            class="truncate font-medium hover:text-primary-color"
                            { "#" (tag.rsplit('/').next().unwrap_or(tag)) }
                            span class="text-zinc-500 text-sm shrink-0" { (count) }
                        }
                    }
                }
            }
        }
    };

    let meta = PageMeta {
        description: Some(format!("All {} tags of the garden", tags.len())),
        ..PageMeta::titled("TAGS")
    };
    render_page_with_meta(app_state, req, &template.into_string(), &meta)
        .respond_to(req)
        .map_into_boxed_body()
}

#[get("/tags/{tag:.*}")]
async fn tag_page(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    tag: web::Path<String>,
) -> HttpResponse {
    if tag.is_empty() {
        return render_tags_index(&app_state, &req).await;
    }

    let posts = app_state.post_search_engine.posts_with_tag(&tag).await;
    if posts.is_empty() {
        return render_not_found(&app_state, &req);
    }

    let template = html! {
        div
        class="p-4 md:p-6 lg:p-8 overflow-auto w-full"
        {
            div
            class="max-w-3xl mx-auto"
            {
                div
                class="mb-8"
                {
                    a
                    href="/tags"
                    class="text-sm text-zinc-500 hover:text-primary-color"
                    { "← All tags" }
                    h1
                    class="pt-4 text-4xl md:text-5xl font-bold text-primary-color mb-2 break-words"
                    { "#" (tag.as_str()) }
                    p
                    class="text-zinc-500"
                    {
                        (posts.len()) @if posts.len() == 1 { " post" } @else { " posts" }
                    }
                }
                ul
                id="tagged-posts"
                class="flex flex-col border-t border-shade-color"
                {
                    @for post in &posts {
                        li
                        class="flex flex-col gap-2 py-4 border-b border-shade-color"
                        {
                            div
                            class="flex flex-wrap items-baseline justify-between gap-x-4"
                            {
                                a
                                href=(format!(
                                    "/posts/{}",
                                    post.file_path.strip_suffix(".md").unwrap_or(&post.file_path)
                                ))
                                class="text-lg font-semibold hover:text-primary-color"
                                { (post.metadata.title) }
                                span class="text-zinc-500 text-sm" { (post.metadata.date) }
                            }
                            p class="text-zinc-400 text-sm" { (post.metadata.description) }
                            div
                            class="flex flex-wrap gap-1"
                            {
                                @for post_tag in &post.metadata.tags {
                                    a
                                    href=(tag_url(post_tag))
                                    class="px-3 py-1 text-xs font-medium bg-primary-color/10 text-primary-color hover:bg-primary-color/25 rounded-full transition-colors"
                                    { "#" (post_tag) }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    let meta = PageMeta {
        description: Some(format!("Notes tagged #{}", tag.as_str())),
        ..PageMeta::titled(format!("#{}", tag.as_str()))
    };
    render_page_with_meta(&app_state, &req, &template.into_string(), &meta)
        .respond_to(&req)
        .map_into_boxed_body()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_urls_percent_encode_anything_but_plain_characters() {
        assert_eq!(tag_url("gaming"), "/tags/gaming");
        assert_eq!(tag_url("digital garden"), "/tags/digital%20garden");
        assert_eq!(tag_url("c#ñ"), "/tags/c%23%C3%B1");
        assert_eq!(tag_url("gaming/horror"), "/tags/gaming/horror");
    }
}
