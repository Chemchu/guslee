use actix_web::{HttpRequest, HttpResponse, Responder, get, web};
use maud::html;

use crate::controllers::{AppState, render_not_found, render_page};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(tag_page);
}

/// Link to the page listing every post with `tag`.
pub fn tag_url(tag: &str) -> String {
    let mut url = String::from("/tags/");
    for byte in tag.bytes() {
        match byte {
            // `/` stays as is so nested tags read as paths: /tags/gaming/horror
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                url.push(byte as char)
            }
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }
    url
}

#[get("/tags/{tag:.*}")]
async fn tag_page(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    tag: web::Path<String>,
) -> HttpResponse {
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
                    h1
                    class="text-4xl md:text-5xl font-bold text-primary-color mb-2 break-words"
                    { "#" (tag.as_str()) }
                    p
                    class="text-zinc-500"
                    {
                        (posts.len()) @if posts.len() == 1 { " note" } @else { " notes" }
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

    render_page(&app_state, &req, &template.into_string())
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
