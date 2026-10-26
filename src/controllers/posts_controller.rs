use actix_web::{
    HttpRequest, HttpResponse, Responder, get,
    web::{self, Data, Html},
};
use cached::cached;
use markdown::{Constructs, Options, ParseOptions};
use maud::{PreEscaped, html};
use search_engine::{
    links::{LinkIndex, WikiLink, find_wikilinks, markdown_link_target, slugify},
    tags::find_inline_tags,
    types::{DEFAULT_SEARCH_LIMIT, Params},
    utils::Post,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path};

use crate::controllers::{
    AppState, is_partial_request, render_not_found, render_page, tags_controller::tag_url,
    wrap_content_into_full_page,
};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(landing)
        .service(search_post)
        .service(get_post_page)
        .service(post_preview);
}

#[get("/")]
async fn landing(app_state: web::Data<AppState>) -> impl Responder {
    let content = match std::fs::read_to_string(format!("{}/welcome.md", app_state.garden_path)) {
        Ok(content) => content,
        Err(e) => {
            log::error!("Failed to read welcome.md: {}", e);
            return Html::new(wrap_content_into_full_page(
                &app_state.app_name,
                "<p>Error loading welcome page</p>",
            ));
        }
    };

    Html::new(wrap_content_into_full_page(
        &app_state.app_name,
        post_page_shell(
            content,
            "welcome".to_string(),
            app_state.post_search_engine.link_index(),
        )
        .into_string()
        .as_str(),
    ))
}

#[get("/posts/{post:.*}")]
async fn get_post_page(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    route: web::Path<String>,
) -> HttpResponse {
    if !is_safe_post_route(&route) {
        return render_not_found(&app_state, &req);
    }

    match fs::read_to_string(format!("{}/{}.md", app_state.garden_path, route)) {
        Ok(md) => render_page(
            &app_state,
            &req,
            &post_page_shell(
                md,
                route.to_string(),
                app_state.post_search_engine.link_index(),
            )
            .into_string(),
        )
        .respond_to(&req)
        .map_into_boxed_body(),
        Err(_err) => render_not_found(&app_state, &req),
    }
}

/// The post body shown in the popover when hovering a link to it.
#[get("/preview/{post:.*}")]
async fn post_preview(app_state: web::Data<AppState>, route: web::Path<String>) -> HttpResponse {
    let post = app_state
        .post_search_engine
        .get_post(&format!("{route}.md"))
        .await;

    match post {
        Some(post) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(
                post_preview_content(&post.content, app_state.post_search_engine.link_index())
                    .into_string(),
            ),
        None => HttpResponse::NotFound().finish(),
    }
}

fn post_preview_content(md: &str, link_index: &LinkIndex) -> PreEscaped<String> {
    html! {
        div class="prose prose-theme prose-sm max-w-none" {
            // Ids would clash with the page's own headings, so the preview
            // scrolls to its anchors through a data attribute instead
            (PreEscaped(render_post_html(md, link_index, "data-preview-anchor")))
        }
    }
}

fn is_safe_post_route(route: &str) -> bool {
    !route.is_empty()
        && Path::new(route)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[get("/search")]
async fn search_post(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    params: web::Query<Params>,
) -> Html {
    if !is_partial_request(&req) {
        return Html::new(String::from("Only HTMX requests for search engine"));
    }

    Html::new(search_results_content(app_state, params.into_inner()).await)
}

#[cached(
    key = "String",
    convert = r#"{
        format!(
            "{}:{}",
            params.query.clone().unwrap_or("empty".to_string()),
            params.limit.clone().unwrap_or(DEFAULT_SEARCH_LIMIT).value()
        )
    }"#
)]
async fn search_results_content(app_state: web::Data<AppState>, params: Params) -> String {
    let is_empty_query = params.query.as_ref().is_none_or(|query| query.len() < 3);

    let matching_posts = match is_empty_query {
        true => get_default_posts(app_state).await,
        false => app_state.post_search_engine.query_posts(&params).await,
    };

    build_posts_list(matching_posts)
}

async fn get_default_posts(app_state: Data<AppState>) -> Vec<Post> {
    let posts_to_search = [
        "welcome.md",
        "hello.md",
        "garden_styling.md",
        "kilbarrack.md",
        "first_job_in_ireland.md",
        "rathmines.md",
    ];
    let default_posts_string = posts_to_search
        .iter()
        .map(|file_name| format!("'{}'", file_name))
        .collect::<Vec<String>>()
        .join(", ");

    let default_posts = app_state
        .post_search_engine
        .raw_query::<Vec<Post>>(
            format!(
                "SELECT * FROM posts WHERE file_name IN [{}] AND metadata.is_draft = false",
                default_posts_string,
            )
            .as_str(),
        )
        .await;

    let posts_map: HashMap<String, Post> = default_posts
        .into_iter()
        .map(|f| (f.file_name.to_string(), f))
        .collect();

    let ordered_files: Vec<Post> = posts_to_search
        .iter()
        .filter_map(|default_doc| posts_map.get(*default_doc).cloned())
        .collect();

    let all_missing_posts: Vec<Post> = app_state
        .post_search_engine
        .raw_query::<Vec<Post>>(
            format!(
                "SELECT * FROM posts WHERE file_name NOT IN [{}] AND metadata.is_draft = false",
                default_posts_string,
            )
            .as_str(),
        )
        .await;

    [ordered_files, all_missing_posts].concat()
}

fn build_posts_list(matching_posts: Vec<Post>) -> String {
    let mut posts_per_topic: HashMap<String, Vec<Post>> = HashMap::default();
    let mut posts_by_filename: HashMap<String, Post> = HashMap::default();
    for m_post in matching_posts.clone() {
        posts_by_filename.insert(m_post.file_name.to_string(), m_post.clone());
        if let Some(topic) = &m_post.metadata.topic {
            posts_per_topic
                .entry(topic.clone())
                .or_default()
                .push(m_post);
        }
    }

    let mut topics_to_render: Vec<(String, Vec<Post>)> = Vec::new();
    let mut posts_to_render: Vec<Post> = Vec::new();
    let mut rendered_topics: HashSet<String> = HashSet::new();

    for matching_post in matching_posts {
        if let Some(p) = posts_by_filename.get(&matching_post.file_name) {
            if let Some(topic) = &p.metadata.topic {
                if !rendered_topics.contains(topic) {
                    rendered_topics.insert(topic.clone());
                    if let Some(topic_posts) = posts_per_topic.get(topic) {
                        topics_to_render.push((topic.clone(), topic_posts.clone()));
                    }
                }
            } else {
                posts_to_render.push(p.clone());
            }
        }
    }

    let html = html! {
        ul {
            @for (_index, (topic, topic_posts)) in topics_to_render.iter().enumerate() {
                li {
                    details
                    open="true"
                    {
                        summary
                        class="cursor-pointer hover:text-primary-color"
                        {
                            (topic)
                        }
                        ul {
                            @for topic_post in topic_posts {
                                li {
                                    a href=(format!(
                                        "/posts/{}",
                                        topic_post
                                            .file_path
                                            .strip_suffix(".md")
                                            .unwrap_or(&topic_post.file_path)
                                    ))
                                    hx-target="#main-section"
                                    hx-swap="innerHTML transition:true"
                                    title=(topic_post.metadata.title)
                                    class="block pl-3 cursor-pointer hover:text-primary-color overflow-hidden truncate"
                                    {
                                        (topic_post.metadata.title)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            @for p in posts_to_render {
                li {
                    a href=(format!(
                        "/posts/{}",
                        p
                            .file_path
                            .strip_suffix(".md")
                            .unwrap_or(&p.file_path)
                    ))
                    hx-target="#main-section"
                    hx-swap="innerHTML transition:true"
                    title=(p.metadata.title)
                    class="block cursor-pointer hover:text-primary-color overflow-hidden truncate"
                    {
                        (p.metadata.title)
                    }
                }
            }
        }
    };
    html.into_string()
}

fn post_url(href: &str) -> Option<String> {
    markdown_link_target(href).map(|target| target.url())
}

fn absolutize_post_links(html: &str) -> String {
    const HREF: &str = "<a href=\"";
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(HREF) {
        let value_start = start + HREF.len();
        let Some(value_len) = rest[value_start..].find('"') else {
            break;
        };
        let href = &rest[value_start..value_start + value_len];

        result.push_str(&rest[..value_start]);
        result.push_str(&post_url(href).unwrap_or_else(|| href.to_string()));
        rest = &rest[value_start + value_len..];
    }

    result.push_str(rest);
    result
}

/// Turns `[[wikilinks]]` in rendered post HTML into links, leaving code alone.
/// Links to notes that don't exist yet are shown faded, like in Obsidian.
fn render_wikilinks(html: &str, link_index: &LinkIndex) -> String {
    const CODE_END: &str = "</code>";
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    loop {
        let code_start = rest.find("<code").unwrap_or(rest.len());
        result.push_str(&render_wikilinks_in_text(&rest[..code_start], link_index));
        rest = &rest[code_start..];
        if rest.is_empty() {
            break;
        }

        let code_end = rest
            .find(CODE_END)
            .map_or(rest.len(), |end| end + CODE_END.len());
        result.push_str(&rest[..code_end]);
        rest = &rest[code_end..];
    }

    result
}

fn render_wikilinks_in_text(text: &str, link_index: &LinkIndex) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last = 0;

    // Embeds (`![[note]]`) aren't supported yet, so they stay as written
    for found in find_wikilinks(text).into_iter().filter(|m| !m.is_embed) {
        let inner = unescape_html(found.inner);
        let link = WikiLink::parse(&inner);
        let href = match link.target {
            "" => link.heading_anchor().map(|anchor| format!("#{anchor}")),
            _ => link_index.wikilink_target(&link).map(|target| target.url()),
        };

        let rendered = match href {
            Some(href) => html! { a href=(href) { (link.display_text()) } },
            None => html! {
                span
                class="wikilink-unresolved"
                title="This note hasn't been written yet"
                { (link.display_text()) }
            },
        };
        result.push_str(&text[last..found.start]);
        result.push_str(&rendered.into_string());
        last = found.end;
    }

    result.push_str(&text[last..]);
    result
}

/// Turns inline `#tags` in rendered post HTML into links to their tag page.
/// Code, links and the inside of HTML tags are left alone.
fn render_inline_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(open) = rest.find('<') {
        result.push_str(&render_inline_tags_in_text(&rest[..open]));
        rest = &rest[open..];

        let name_len = rest[1..]
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(rest.len() - 1);
        let skipped_element = matches!(&rest[1..1 + name_len], "a" | "code" | "pre");
        let end = match skipped_element {
            true => {
                let closing = format!("</{}>", &rest[1..1 + name_len]);
                rest.find(&closing).map(|i| i + closing.len())
            }
            false => rest.find('>').map(|i| i + 1),
        }
        .unwrap_or(rest.len());

        result.push_str(&rest[..end]);
        rest = &rest[end..];
    }

    result.push_str(&render_inline_tags_in_text(rest));
    result
}

fn render_inline_tags_in_text(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last = 0;

    for tag in find_inline_tags(text) {
        result.push_str(&text[last..tag.start]);
        result.push_str(
            &html! {
                a
                href=(tag_url(tag.name))
                class="px-1.5 py-0.5 text-[0.9em] no-underline font-medium bg-primary-color/10 text-primary-color hover:bg-primary-color/25 rounded-full transition-colors"
                { "#" (tag.name) }
            }
            .into_string(),
        );
        last = tag.end;
    }

    result.push_str(&text[last..]);
    result
}

fn unescape_html(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// Gives every heading an anchor named like its text, so `notes/fog#the-fog`
/// and `[[fog#The fog]]` land on it. Repeated headings get `-1`, `-2`...
fn add_heading_anchors(html: &str, attribute: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut seen: HashMap<String, usize> = HashMap::new();
    let mut rest = html;

    while let Some(start) = rest.find("<h") {
        let level = rest.as_bytes().get(start + 2).copied();
        let is_heading = level.is_some_and(|l| (b'1'..=b'6').contains(&l))
            && rest.as_bytes().get(start + 3) == Some(&b'>');
        if !is_heading {
            result.push_str(&rest[..start + 2]);
            rest = &rest[start + 2..];
            continue;
        }

        let tag = &rest[start + 1..start + 3];
        let content_start = start + 4;
        let Some(content_len) = rest[content_start..].find(&format!("</{tag}>")) else {
            break;
        };
        let text = unescape_html(&strip_tags(
            &rest[content_start..content_start + content_len],
        ));
        let mut anchor = slugify(&text);
        let count = seen.entry(anchor.clone()).or_default();
        if *count > 0 {
            anchor = format!("{anchor}-{count}");
        }
        *count += 1;

        result.push_str(&rest[..start]);
        result.push_str(&format!("<{tag} {attribute}=\"{anchor}\">"));
        rest = &rest[content_start..];
    }

    result.push_str(rest);
    result
}

fn strip_tags(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => text.push(c),
            _ => {}
        }
    }
    text
}

fn render_post_html(md: &str, link_index: &LinkIndex, anchor_attribute: &str) -> String {
    let frontmatter = Options {
        parse: ParseOptions {
            constructs: Constructs {
                frontmatter: true,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let html = markdown::to_html_with_options(md, &frontmatter).unwrap();
    let html = render_inline_tags(&render_wikilinks(&absolutize_post_links(&html), link_index));
    add_heading_anchors(&html, anchor_attribute)
}

fn post_page_shell(md: String, post_path: String, link_index: &LinkIndex) -> PreEscaped<String> {
    html! {
        div
        class="prose prose-theme w-full max-w-full p-4 md:p-6 lg:p-8 overflow-auto text-sm md:text-base"
            {
               (PreEscaped(render_post_html(&md, link_index, "id")))
            }
        div
        id="right-section-resizer"
        data-resizer
        data-resize-target="right-section"
        data-resize-var="--right-section-width"
        data-resize-edge="left"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize side panel"
        title="Drag to resize, double-click to reset"
        class="hidden lg:block w-1 shrink-0 cursor-col-resize bg-shade-color hover:bg-primary-color transition-colors"
        {}
        div
        id="right-section"
        class="hidden lg:flex flex-col lg:w-[var(--right-section-width,20rem)] shrink-0 h-full"
        {
            div
            class="flex flex-col grow w-full min-h-10 max-h-[34vh] border-b border-shade-color"
            {
                div
                id="upper-right-section"
                hx-get=(format!("/graph/{}", post_path))
                hx-trigger="load"
                hx-target="#upper-right-section"
                hx-swap="innerHTML"
                class="flex w-full grow cursor-grab active:cursor-grabbing h-full"
                {}
            }
            div
            class="flex flex-col grow w-full min-h-0 overflow-y-auto"
            {
                div
                id="bottom-right-section"
                hx-get=(format!("/metadata/{}", post_path))
                hx-trigger="load"
                hx-target="#bottom-right-section"
                hx-swap="innerHTML"
                class="flex w-full grow p-2"
                {
                    "Loading..."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use search_engine::utils::MdMetadata;

    fn index() -> LinkIndex {
        LinkIndex::new(["welcome.md", "notes/fog.md", "journal/2025/rock & roll.md"])
    }

    fn post(file_path: &str, title: &str, topic: Option<&str>) -> Post {
        Post {
            file_name: file_path.rsplit('/').next().unwrap().to_string(),
            file_path: file_path.to_string(),
            metadata: MdMetadata {
                title: title.to_string(),
                topic: topic.map(str::to_string),
                description: String::new(),
                tags: vec![],
                date: String::new(),
                is_draft: false,
                post_source_url: String::new(),
                reading_time: 1,
            },
            content: String::new(),
        }
    }

    #[test]
    fn plain_and_nested_post_routes_are_allowed() {
        assert!(is_safe_post_route("welcome"));
        assert!(is_safe_post_route("journal/2025/rathmines"));
    }

    #[test]
    fn routes_escaping_the_garden_are_rejected() {
        assert!(!is_safe_post_route("../README"));
        assert!(!is_safe_post_route("journal/../../secrets"));
        assert!(!is_safe_post_route("/etc/passwd"));
        assert!(!is_safe_post_route("./welcome"));
        assert!(!is_safe_post_route(""));
    }

    #[test]
    fn posts_with_a_topic_are_grouped_under_it() {
        let html = build_posts_list(vec![
            post("gaming/a.md", "Game A", Some("Gaming")),
            post("welcome.md", "Welcome", None),
            post("gaming/b.md", "Game B", Some("Gaming")),
        ]);

        assert_eq!(html.matches("<summary").count(), 1);
        assert!(html.contains(">Gaming</summary>"));
        let a = html.find("Game A").unwrap();
        let b = html.find("Game B").unwrap();
        let welcome = html.find(">Welcome<").unwrap();
        assert!(a < b && b < welcome, "topics come first, in result order");
    }

    #[test]
    fn post_links_drop_the_markdown_extension_and_target_the_main_section() {
        let html = build_posts_list(vec![post("journal/2025/x.md", "X", None)]);

        assert!(html.contains("href=\"/posts/journal/2025/x\""));
        assert!(html.contains("hx-target=\"#main-section\""));
    }

    #[test]
    fn titles_are_html_escaped() {
        let html = build_posts_list(vec![post("x.md", "<script>alert(1)</script>", None)]);

        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn garden_relative_links_become_absolute_post_urls() {
        assert_eq!(post_url("hello").as_deref(), Some("/posts/hello"));
        assert_eq!(
            post_url("journal/2025/notes").as_deref(),
            Some("/posts/journal/2025/notes")
        );
        assert_eq!(post_url("./hello.md").as_deref(), Some("/posts/hello"));
        assert_eq!(
            post_url("notes/fog#the-fog").as_deref(),
            Some("/posts/notes/fog#the-fog")
        );
    }

    #[test]
    fn external_absolute_and_anchor_links_are_left_alone() {
        for href in [
            "https://rust-lang.org/",
            "http://example.com",
            "mailto:me@example.com",
            "/news",
            "#section",
            "?query=1",
            "",
        ] {
            assert_eq!(post_url(href), None, "{href:?}");
        }
    }

    #[test]
    fn rendered_links_are_rewritten_in_place() {
        let html = r#"<p><a href="notes/fog">fog</a> and <a href="https://htmx.org/" title="htmx">htmx</a></p>"#;

        assert_eq!(
            absolutize_post_links(html),
            r#"<p><a href="/posts/notes/fog">fog</a> and <a href="https://htmx.org/" title="htmx">htmx</a></p>"#
        );
        assert_eq!(absolutize_post_links("<p>no links</p>"), "<p>no links</p>");
    }

    #[test]
    fn post_shell_links_resolve_from_the_garden_root() {
        let md = "See [the fog](notes/fog) and [Rust](https://rust-lang.org/).".to_string();

        let html =
            post_page_shell(md, "journal/2025/deep_post".to_string(), &index()).into_string();

        assert!(html.contains(r#"<a href="/posts/notes/fog">the fog</a>"#));
        assert!(html.contains(r#"<a href="https://rust-lang.org/">Rust</a>"#));
    }

    #[test]
    fn post_shell_renders_markdown_without_the_frontmatter() {
        let md = "---\ntitle: \"Hidden\"\n---\n# Heading\n\nSome *text*.".to_string();

        let html = post_page_shell(md, "notes/first".to_string(), &index()).into_string();

        assert!(html.contains(r#"<h1 id="heading">Heading</h1>"#));
        assert!(html.contains("<em>text</em>"));
        assert!(!html.contains("title: "));
        assert!(html.contains("hx-get=\"/graph/notes/first\""));
        assert!(html.contains("hx-get=\"/metadata/notes/first\""));
    }

    #[test]
    fn wikilinks_link_to_the_note_they_name() {
        let html = render_wikilinks(
            "<p>[[fog]], [[Fog#The fog|that fog]] and [[#Intro]]</p>",
            &index(),
        );

        assert_eq!(
            html,
            r##"<p><a href="/posts/notes/fog">fog</a>, <a href="/posts/notes/fog#the-fog">that fog</a> and <a href="#intro">Intro</a></p>"##
        );
    }

    #[test]
    fn missing_notes_render_as_unresolved_links() {
        let html = render_wikilinks("<p>[[not yet]]</p>", &index());

        assert_eq!(
            html,
            r#"<p><span class="wikilink-unresolved" title="This note hasn't been written yet">not yet</span></p>"#
        );
    }

    #[test]
    fn wikilinks_in_code_and_embeds_are_left_alone() {
        let html = "<pre><code>[[fog]]</code></pre><p><code>[[fog]]</code> ![[fog]]</p>";

        assert_eq!(render_wikilinks(html, &index()), html);
    }

    #[test]
    fn wikilink_text_is_unescaped_to_resolve_and_escaped_again() {
        let html = render_wikilinks("<p>[[rock &amp; roll|R&amp;R &lt;3]]</p>", &index());

        assert_eq!(
            html,
            r#"<p><a href="/posts/journal/2025/rock &amp; roll">R&amp;R &lt;3</a></p>"#
        );
    }

    #[test]
    fn post_shell_renders_wikilinks() {
        let md = "Read [[fog]] and `[[fog]]`.".to_string();

        let html = post_page_shell(md, "welcome".to_string(), &index()).into_string();

        assert!(html.contains(r#"<a href="/posts/notes/fog">fog</a>"#));
        assert!(html.contains("<code>[[fog]]</code>"));
    }

    #[test]
    fn headings_get_anchors_named_after_their_text() {
        let html = "<h1>The fog</h1><p>x</p><h2>Año <em>nuevo</em> &amp; more</h2><hr />";

        assert_eq!(
            add_heading_anchors(html, "id"),
            r#"<h1 id="the-fog">The fog</h1><p>x</p><h2 id="año-nuevo-more">Año <em>nuevo</em> &amp; more</h2><hr />"#
        );
    }

    #[test]
    fn repeated_headings_get_numbered_anchors() {
        let html = add_heading_anchors("<h2>Notes</h2><h3>Notes</h3><h2>Notes</h2>", "id");

        assert_eq!(
            html,
            r#"<h2 id="notes">Notes</h2><h3 id="notes-1">Notes</h3><h2 id="notes-2">Notes</h2>"#
        );
    }

    #[test]
    fn heading_links_match_heading_anchors() {
        let md = "# The fog\n\nSee [[#The fog]].".to_string();

        let html = post_page_shell(md, "notes/fog".to_string(), &index()).into_string();

        assert!(html.contains(r#"<h1 id="the-fog">"#));
        assert!(html.contains(r##"<a href="#the-fog">The fog</a>"##));
    }

    #[test]
    fn previews_use_data_anchors_instead_of_ids() {
        let html = post_preview_content("---\ntitle: \"x\"\n---\n# The fog\n\n[[fog]]", &index())
            .into_string();

        assert!(html.contains(r#"<h1 data-preview-anchor="the-fog">The fog</h1>"#));
        assert!(html.contains(r#"<a href="/posts/notes/fog">fog</a>"#));
        assert!(!html.contains(" id="));
        assert!(!html.contains("title:"));
    }

    #[test]
    fn inline_tags_link_to_their_tag_page() {
        let html = render_inline_tags("<p>#intro and #gaming/horror, colour #2F3E46</p>");

        assert!(html.starts_with(r#"<p><a href="/tags/intro" class="#));
        assert!(html.contains(r#"href="/tags/gaming/horror""#));
        assert!(html.contains(">#gaming/horror</a>, colour #2F3E46</p>"));
    }

    #[test]
    fn inline_tags_in_code_links_and_attributes_are_left_alone() {
        let html = r##"<pre><code>#fenced</code></pre><p><code>#code</code> <a href="#top" title="x #y">#link</a> <abbr>#abbr</abbr></p>"##;

        let rendered = render_inline_tags(html);

        assert!(rendered.starts_with(
            r##"<pre><code>#fenced</code></pre><p><code>#code</code> <a href="#top" title="x #y">#link</a> <abbr><a href="/tags/abbr""##
        ));
    }

    #[test]
    fn post_shell_renders_inline_tags_but_not_headings() {
        let md = "# Title\n\nText #intro.".to_string();

        let html = post_page_shell(md, "welcome".to_string(), &index()).into_string();

        assert!(html.contains(r#"<h1 id="title">Title</h1>"#));
        assert!(html.contains(r#"href="/tags/intro""#));
    }
}
