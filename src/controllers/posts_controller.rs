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
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path};

use crate::controllers::obsidian_markdown::{render_callouts, render_highlights};
use crate::controllers::{
    AppState, PageMeta, base_url, is_partial_request, render_not_found, render_page_with_meta,
    tags_controller::tag_url, wrap_content_into_full_page, wrap_content_with_meta,
};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(landing)
        .service(search_post)
        .service(get_post_page)
        .service(post_preview)
        .service(switcher_posts);
}

#[get("/")]
async fn landing(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
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

    let meta = PageMeta {
        title: None,
        ..post_meta(&app_state, "welcome").await
    };
    let url = format!("{}/", base_url(&req));
    Html::new(wrap_content_with_meta(
        &app_state.app_name,
        post_page_shell(content, "welcome".to_string(), &Garden::of(&app_state))
            .into_string()
            .as_str(),
        &meta,
        Some(&url),
    ))
}

async fn post_meta(app_state: &AppState, route: &str) -> PageMeta {
    match app_state
        .post_search_engine
        .get_post(&format!("{route}.md"))
        .await
    {
        Some(post) => PageMeta {
            title: Some(post.metadata.title),
            description: Some(post.metadata.description).filter(|d| !d.is_empty()),
            is_article: true,
        },
        None => PageMeta::default(),
    }
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
        Ok(md) => render_page_with_meta(
            &app_state,
            &req,
            &post_page_shell(md, route.to_string(), &Garden::of(&app_state)).into_string(),
            &post_meta(&app_state, &route).await,
        )
        .respond_to(&req)
        .map_into_boxed_body(),
        Err(_err) => render_not_found(&app_state, &req),
    }
}

#[get("/preview/{post:.*}")]
async fn post_preview(app_state: web::Data<AppState>, route: web::Path<String>) -> HttpResponse {
    let post = app_state
        .post_search_engine
        .get_post(&format!("{route}.md"))
        .await;

    match post {
        Some(post) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(post_preview_content(&post.content, &Garden::of(&app_state)).into_string()),
        None => HttpResponse::NotFound().finish(),
    }
}

fn post_preview_content(md: &str, garden: &Garden) -> PreEscaped<String> {
    html! {
        div class="prose prose-theme prose-sm max-w-none" {
            (PreEscaped(render_post_html(md, garden, "data-preview-anchor").0))
        }
    }
}

#[derive(Serialize)]
struct SwitcherPost {
    title: String,
    path: String,
}

#[get("/switcher.json")]
async fn switcher_posts(app_state: web::Data<AppState>) -> HttpResponse {
    let posts: Vec<SwitcherPost> = app_state
        .post_search_engine
        .published_posts()
        .await
        .into_iter()
        .map(|post| SwitcherPost {
            path: post
                .file_path
                .strip_suffix(".md")
                .unwrap_or(&post.file_path)
                .to_string(),
            title: post.metadata.title,
        })
        .collect();

    HttpResponse::Ok().json(posts)
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
            "{}:{}:{}",
            params.query.clone().unwrap_or("empty".to_string()),
            params.limit.clone().unwrap_or(DEFAULT_SEARCH_LIMIT).value(),
            params.view.clone().unwrap_or_default()
        )
    }"#
)]
async fn search_results_content(app_state: web::Data<AppState>, params: Params) -> String {
    let is_empty_query = params.query.as_ref().is_none_or(|query| query.len() < 3);
    if is_empty_query && params.view.as_deref() == Some("folders") {
        return build_folder_tree(&app_state.post_search_engine.published_posts().await);
    }

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

#[derive(Default)]
struct Folder<'a> {
    folders: BTreeMap<String, Folder<'a>>,
    posts: Vec<&'a Post>,
}

fn build_folder_tree(posts: &[Post]) -> String {
    let mut root = Folder::default();
    for post in posts {
        let mut folder = &mut root;
        if let Some((dirs, _)) = post.file_path.rsplit_once('/') {
            for dir in dirs.split('/') {
                folder = folder.folders.entry(dir.to_string()).or_default();
            }
        }
        folder.posts.push(post);
    }

    render_folder(&root, "").into_string()
}

fn render_folder(folder: &Folder, path: &str) -> PreEscaped<String> {
    let mut folders: Vec<(&String, &Folder)> = folder.folders.iter().collect();
    folders.sort_by_key(|(name, _)| name.to_lowercase());
    let mut posts = folder.posts.clone();
    posts.sort_by_key(|post| post.metadata.title.to_lowercase());

    html! {
        ul
        id=[path.is_empty().then_some("file-tree")]
        class=(if path.is_empty() { "flex flex-col" } else { "flex flex-col ml-1 pl-2 border-l border-shade-color" })
        {
            @for (name, subfolder) in folders {
                @let folder_path = if path.is_empty() { name.clone() } else { format!("{path}/{name}") };
                li {
                    details data-folder=(folder_path) {
                        summary class="cursor-pointer hover:text-primary-color truncate" { (name) }
                        (render_folder(subfolder, &folder_path))
                    }
                }
            }
            @for post in posts {
                li {
                    a
                    href=(format!(
                        "/posts/{}",
                        post.file_path.strip_suffix(".md").unwrap_or(&post.file_path)
                    ))
                    hx-target="#main-section"
                    hx-swap="innerHTML transition:true"
                    title=(post.metadata.title)
                    class="block truncate cursor-pointer hover:text-primary-color aria-[current=page]:text-primary-color"
                    { (post.metadata.title) }
                }
            }
        }
    }
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
                                    class="block pl-3 cursor-pointer hover:text-primary-color overflow-hidden truncate aria-[current=page]:text-primary-color"
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
                    class="block cursor-pointer hover:text-primary-color overflow-hidden truncate aria-[current=page]:text-primary-color"
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

    for found in find_wikilinks(text) {
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

fn add_heading_anchors(html: &str, attribute: &str) -> (String, Vec<Heading>) {
    let mut headings = Vec::new();
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

        headings.push(Heading {
            level: level.map_or(1, |l| l - b'0'),
            anchor: anchor.clone(),
            text: text.trim().to_string(),
        });
        result.push_str(&rest[..start]);
        result.push_str(&format!("<{tag} {attribute}=\"{anchor}\">"));
        rest = &rest[content_start..];
    }

    result.push_str(rest);
    (result, headings)
}

#[derive(Debug, PartialEq, Eq)]
struct Heading {
    level: u8,
    anchor: String,
    text: String,
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

pub struct Garden<'a> {
    pub link_index: &'a LinkIndex,
    pub path: &'a str,
}

impl<'a> Garden<'a> {
    fn of(app_state: &'a AppState) -> Garden<'a> {
        Garden {
            link_index: app_state.post_search_engine.link_index(),
            path: &app_state.garden_path,
        }
    }
}

const MAX_EMBED_DEPTH: usize = 2;
const EMBED_ANCHOR: &str = "data-embed-anchor";

fn render_embeds(html: &str, garden: &Garden, depth: usize) -> String {
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find("<p>") {
        let content_start = start + "<p>".len();
        let Some(content_len) = rest[content_start..].find("</p>") else {
            break;
        };
        let end = content_start + content_len + "</p>".len();
        let content = rest[content_start..content_start + content_len].trim();

        let embed = match find_wikilinks(content).as_slice() {
            [found] if found.is_embed && found.start == 0 && found.end == content.len() => {
                embed_block(found.inner, garden, depth)
            }
            _ => None,
        };
        result.push_str(&rest[..start]);
        result.push_str(embed.as_deref().unwrap_or(&rest[start..end]));
        rest = &rest[end..];
    }

    result.push_str(rest);
    result
}

fn embed_block(inner: &str, garden: &Garden, depth: usize) -> Option<String> {
    if depth >= MAX_EMBED_DEPTH {
        return None;
    }
    let inner = unescape_html(inner);
    let link = WikiLink::parse(&inner);
    let target = garden.link_index.wikilink_target(&link)?;
    let md = fs::read_to_string(format!("{}/{}", garden.path, target.file_path())).ok()?;

    let (html, _) = render_post_html_at_depth(&md, garden, EMBED_ANCHOR, depth + 1);
    let content = match &target.fragment {
        Some(anchor) => heading_section(&html, anchor)?,
        None => html,
    };

    Some(
        html! {
            div class="embed" {
                a class="embed-link" href=(target.url()) { (link.display_text()) }
                div class="embed-content" { (PreEscaped(content)) }
            }
        }
        .into_string(),
    )
}

fn heading_section(html: &str, anchor: &str) -> Option<String> {
    let marker = format!(" {EMBED_ANCHOR}=\"{anchor}\">");
    let marker_at = html.find(&marker)?;
    let start = html[..marker_at].rfind("<h")?;
    let level = html.as_bytes()[start + 2];

    let mut end = html.len();
    let mut search = marker_at;
    while let Some(found) = html[search..].find("<h") {
        let at = search + found;
        let next_level = html.as_bytes().get(at + 2).copied();
        let is_heading = html.as_bytes().get(at + 3) == Some(&b' ');
        if is_heading && next_level.is_some_and(|l| (b'1'..=level).contains(&l)) {
            end = at;
            break;
        }
        search = at + 2;
    }

    Some(html[start..end].to_string())
}

fn render_post_html(md: &str, garden: &Garden, anchor_attribute: &str) -> (String, Vec<Heading>) {
    render_post_html_at_depth(md, garden, anchor_attribute, 0)
}

fn render_post_html_at_depth(
    md: &str,
    garden: &Garden,
    anchor_attribute: &str,
    embed_depth: usize,
) -> (String, Vec<Heading>) {
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
    let html = render_callouts(&absolutize_post_links(&html));
    let html = render_embeds(&html, garden, embed_depth);
    let html = render_wikilinks(&html, garden.link_index);
    let html = render_inline_tags(&render_highlights(&html));
    add_heading_anchors(&html, anchor_attribute)
}

const OUTLINE_INDENT: [&str; 6] = ["pl-0", "pl-3", "pl-6", "pl-9", "pl-12", "pl-15"];

fn outline(headings: &[Heading]) -> PreEscaped<String> {
    let top_level = headings.iter().map(|h| h.level).min().unwrap_or(1);

    html! {
        @if headings.is_empty() {
            p class="text-zinc-500 text-sm" { "No headings in this note." }
        } @else {
            ul class="flex flex-col gap-1 text-sm" {
                @for heading in headings {
                    li class=(OUTLINE_INDENT[usize::from(heading.level - top_level)]) {
                        a
                        href=(format!("#{}", heading.anchor))
                        title=(heading.text)
                        class="block truncate text-zinc-300 hover:text-primary-color aria-[current=true]:text-primary-color transition-colors"
                        { (heading.text) }
                    }
                }
            }
        }
    }
}

fn post_page_shell(md: String, post_path: String, garden: &Garden) -> PreEscaped<String> {
    let (post_html, headings) = render_post_html(&md, garden, "id");
    let tab_class = "pb-1 border-b-2 border-transparent text-zinc-500 hover:text-bright-color aria-selected:text-primary-color aria-selected:border-primary-color uppercase tracking-wider transition-colors";

    html! {
        div
        class="prose prose-theme w-full max-w-full p-4 md:p-6 lg:p-8 overflow-auto text-sm md:text-base"
            {
               (PreEscaped(post_html))
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
            class="relative w-full h-[34vh] shrink-0 border-b border-shade-color"
            {
                div
                id="upper-right-section"
                hx-get=(format!("/graph/{}", post_path))
                hx-vals="js:{depth: window.graphDepth ? window.graphDepth() : 1}"
                hx-trigger="load"
                hx-target="#upper-right-section"
                hx-swap="innerHTML"
                class="absolute inset-0 cursor-grab active:cursor-grabbing"
                {}
            }
            div
            class="flex flex-col grow w-full min-h-0"
            {
                div
                role="tablist"
                aria-label="Side panel"
                class="flex gap-4 px-2 pt-2 text-sm font-semibold border-b border-shade-color"
                {
                    button
                    type="button"
                    role="tab"
                    data-panel-tab="metadata"
                    aria-controls="bottom-right-section"
                    aria-selected="true"
                    class=(tab_class)
                    { "Metadata" }
                    button
                    type="button"
                    role="tab"
                    data-panel-tab="outline"
                    aria-controls="outline-section"
                    aria-selected="false"
                    class=(tab_class)
                    { "Outline" }
                }
                div
                class="flex flex-col grow min-h-0 overflow-y-auto"
                {
                    div
                    id="bottom-right-section"
                    role="tabpanel"
                    data-panel="metadata"
                    hx-get=(format!("/metadata/{}", post_path))
                    hx-trigger="load"
                    hx-target="#bottom-right-section"
                    hx-swap="innerHTML"
                    class="flex w-full grow p-2"
                    {
                        "Loading..."
                    }
                    nav
                    id="outline-section"
                    role="tabpanel"
                    data-panel="outline"
                    aria-label="Outline"
                    hidden
                    class="w-full p-2"
                    {
                        (outline(&headings))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use search_engine::utils::MdMetadata;

    fn garden() -> Garden<'static> {
        static INDEX: std::sync::OnceLock<LinkIndex> = std::sync::OnceLock::new();
        Garden {
            link_index: INDEX.get_or_init(index),
            path: concat!(env!("CARGO_MANIFEST_DIR"), "/crates/e2e/fixtures/garden"),
        }
    }

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
                last_updated: None,
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
            post_page_shell(md, "journal/2025/deep_post".to_string(), &garden()).into_string();

        assert!(html.contains(r#"<a href="/posts/notes/fog">the fog</a>"#));
        assert!(html.contains(r#"<a href="https://rust-lang.org/">Rust</a>"#));
    }

    #[test]
    fn post_shell_renders_markdown_without_the_frontmatter() {
        let md = "---\ntitle: \"Hidden\"\n---\n# Heading\n\nSome *text*.".to_string();

        let html = post_page_shell(md, "notes/first".to_string(), &garden()).into_string();

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
    fn wikilinks_in_code_are_left_alone() {
        let html = "<pre><code>[[fog]]</code></pre><p><code>[[fog]]</code></p>";

        assert_eq!(render_wikilinks(html, &index()), html);
    }

    #[test]
    fn embeds_inside_a_sentence_become_links() {
        assert_eq!(
            render_wikilinks("<p>See ![[fog]] here</p>", &index()),
            r#"<p>See <a href="/posts/notes/fog">fog</a> here</p>"#
        );
    }

    #[test]
    fn embeds_on_their_own_line_show_the_note() {
        let html = render_embeds("<p>Intro</p>\n<p>![[fog|The fog note]]</p>", &garden(), 0);

        assert!(html.starts_with(
            r#"<p>Intro</p>
<div class="embed"><a class="embed-link" href="/posts/notes/fog">The fog note</a><div class="embed-content">"#
        ));
        assert!(html.contains(r#"<h1 data-embed-anchor="the-fog">The fog</h1>"#));
        assert!(html.contains("never lifts in Silent Hill"));
        assert!(!html.contains("title:"), "front matter is not shown");
    }

    #[test]
    fn heading_embeds_show_only_that_section() {
        let html = render_embeds("<p>![[fog#Getting around]]</p>", &garden(), 0);

        assert!(
            html.contains(r#"href="/posts/notes/fog#getting-around">fog &gt; Getting around</a>"#)
        );
        assert!(html.contains(r#"<h2 data-embed-anchor="getting-around">"#));
        assert!(html.contains("Bring a radio"));
        assert!(
            !html.contains("never lifts"),
            "text before the heading is left out"
        );
    }

    #[test]
    fn sections_end_at_the_next_heading_of_the_same_level() {
        let html = concat!(
            r#"<h1 data-embed-anchor="a">A</h1><p>a</p>"#,
            r#"<h2 data-embed-anchor="b">B</h2><p>b</p><h3 data-embed-anchor="c">C</h3><p>c</p>"#,
            r#"<h2 data-embed-anchor="d">D</h2><p>d</p>"#
        );

        assert_eq!(
            heading_section(html, "b").unwrap(),
            r#"<h2 data-embed-anchor="b">B</h2><p>b</p><h3 data-embed-anchor="c">C</h3><p>c</p>"#
        );
        assert!(heading_section(html, "a").unwrap().ends_with("<p>d</p>"));
        assert_eq!(heading_section(html, "missing"), None);
    }

    #[test]
    fn missing_notes_sections_and_deep_embeds_fall_back_to_links() {
        for (html, depth) in [
            ("<p>![[ghosts]]</p>", 0),
            ("<p>![[fog#No such heading]]</p>", 0),
            ("<p>![[fog]]</p>", MAX_EMBED_DEPTH),
        ] {
            assert_eq!(render_embeds(html, &garden(), depth), html);
        }
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

        let html = post_page_shell(md, "welcome".to_string(), &garden()).into_string();

        assert!(html.contains(r#"<a href="/posts/notes/fog">fog</a>"#));
        assert!(html.contains("<code>[[fog]]</code>"));
    }

    #[test]
    fn headings_get_anchors_named_after_their_text() {
        let html = "<h1>The fog</h1><p>x</p><h2>Año <em>nuevo</em> &amp; more</h2><hr />";

        assert_eq!(
            add_heading_anchors(html, "id").0,
            r#"<h1 id="the-fog">The fog</h1><p>x</p><h2 id="año-nuevo-more">Año <em>nuevo</em> &amp; more</h2><hr />"#
        );
    }

    #[test]
    fn repeated_headings_get_numbered_anchors() {
        let (html, _) = add_heading_anchors("<h2>Notes</h2><h3>Notes</h3><h2>Notes</h2>", "id");

        assert_eq!(
            html,
            r#"<h2 id="notes">Notes</h2><h3 id="notes-1">Notes</h3><h2 id="notes-2">Notes</h2>"#
        );
    }

    #[test]
    fn heading_links_match_heading_anchors() {
        let md = "# The fog\n\nSee [[#The fog]].".to_string();

        let html = post_page_shell(md, "notes/fog".to_string(), &garden()).into_string();

        assert!(html.contains(r#"<h1 id="the-fog">"#));
        assert!(html.contains(r##"<a href="#the-fog">The fog</a>"##));
    }

    #[test]
    fn previews_use_data_anchors_instead_of_ids() {
        let html = post_preview_content("---\ntitle: \"x\"\n---\n# The fog\n\n[[fog]]", &garden())
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

        let html = post_page_shell(md, "welcome".to_string(), &garden()).into_string();

        assert!(html.contains(r#"<h1 id="title">Title</h1>"#));
        assert!(html.contains(r#"href="/tags/intro""#));
    }

    #[test]
    fn headings_are_collected_for_the_outline() {
        let (_, headings) = add_heading_anchors("<h2>A <em>b</em></h2><h3>A b</h3>", "id");

        assert_eq!(
            headings,
            vec![
                Heading {
                    level: 2,
                    anchor: "a-b".to_string(),
                    text: "A b".to_string()
                },
                Heading {
                    level: 3,
                    anchor: "a-b-1".to_string(),
                    text: "A b".to_string()
                },
            ]
        );
    }

    #[test]
    fn outline_indents_from_the_top_heading_level() {
        let md = "## Top\n\n### Child\n\n## Next".to_string();

        let html = post_page_shell(md, "welcome".to_string(), &garden()).into_string();
        let outline = &html[html.find("id=\"outline-section\"").unwrap()..];

        assert!(outline.contains(r##"<li class="pl-0"><a href="#top""##));
        assert!(outline.contains(r##"<li class="pl-3"><a href="#child""##));
        assert!(outline.contains(r##"<li class="pl-0"><a href="#next""##));
    }

    #[test]
    fn notes_without_headings_say_so_in_the_outline() {
        let html = post_page_shell("Just text.".to_string(), "welcome".to_string(), &garden())
            .into_string();

        assert!(html.contains("No headings in this note."));
    }

    #[test]
    fn post_shell_renders_callouts_and_highlights() {
        let md = "> [!tip] Hint\n> Read ==this== and [[fog]].".to_string();

        let html = post_page_shell(md, "welcome".to_string(), &garden()).into_string();

        assert!(html.contains(
            r#"<div class="callout" data-callout="tip"><div class="callout-title">Hint</div>"#
        ));
        assert!(html.contains(r#"Read <mark>this</mark> and <a href="/posts/notes/fog">fog</a>."#));
    }

    #[test]
    fn folder_tree_lists_folders_first_then_posts_by_title() {
        let html = build_folder_tree(&[
            post("welcome.md", "Welcome", None),
            post("journal/2025/b.md", "Zebra", None),
            post("journal/2025/a.md", "Apple", None),
            post("journal/old.md", "Old times", None),
            post("Gaming/x.md", "X", None),
            post("about.md", "About", None),
        ]);

        let order: Vec<usize> = [
            r#"data-folder="Gaming""#,
            r#"href="/posts/Gaming/x""#,
            r#"data-folder="journal""#,
            r#"data-folder="journal/2025""#,
            ">Apple<",
            ">Zebra<",
            ">Old times<",
            ">About<",
            ">Welcome<",
        ]
        .iter()
        .map(|needle| {
            html.find(needle)
                .unwrap_or_else(|| panic!("{needle} missing"))
        })
        .collect();
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{html}");
        assert!(html.starts_with(r#"<ul id="file-tree""#));
        assert_eq!(html.matches("id=\"file-tree\"").count(), 1);
        assert!(html.contains(r##"hx-target="#main-section""##));
    }

    #[test]
    fn rendering_never_splits_multibyte_characters() {
        let pieces = [
            "–",
            "é",
            "日本",
            "🌫️",
            "==",
            "==ñ==",
            "[[fog]]",
            "![[fog]]",
            "[[ñ]]",
            "#tág",
            "#",
            "`ç`",
            "**ü**",
            "<",
            "&",
            "> [!note] Tí",
            "\n",
            "# Hé",
            "[l](fog)",
            " ",
        ];
        for a in pieces {
            for b in pieces {
                for c in pieces {
                    let md = format!("{a}{b}{c}");
                    let _ = render_post_html(&md, &garden(), "id");
                }
            }
        }
    }
}
