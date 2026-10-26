use actix_web::{
    HttpRequest, HttpResponse, Responder, get,
    web::{self, Data, Html},
};
use cached::cached;
use markdown::{Constructs, Options, ParseOptions};
use maud::{PreEscaped, html};
use search_engine::{
    types::{DEFAULT_SEARCH_LIMIT, Params},
    utils::Post,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path};

use crate::controllers::{
    AppState, is_partial_request, render_not_found, render_page, wrap_content_into_full_page,
};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(landing)
        .service(search_post)
        .service(get_post_page);
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
        post_page_shell(content, "welcome".to_string())
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
            &post_page_shell(md, route.to_string()).into_string(),
        )
        .respond_to(&req)
        .map_into_boxed_body(),
        Err(_err) => render_not_found(&app_state, &req),
    }
}

// Only plain path segments are allowed so `..` (or an absolute path) cannot escape the
// garden directory
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

fn post_page_shell(md: String, post_path: String) -> PreEscaped<String> {
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

    html! {
        div
        class="prose prose-theme w-full max-w-full p-4 md:p-6 lg:p-8 overflow-auto text-sm md:text-base"
            {
               (PreEscaped(markdown::to_html_with_options(&md, &frontmatter).unwrap()))
            }
        div
        id="right-section"
        class="hidden lg:flex flex-col min-w-[16rem] xl:min-w-[20rem] max-w-md h-full border-l border-shade-color"
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
            class="flex flex-col grow w-full"
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
    fn post_shell_renders_markdown_without_the_frontmatter() {
        let md = "---\ntitle: \"Hidden\"\n---\n# Heading\n\nSome *text*.".to_string();

        let html = post_page_shell(md, "notes/first".to_string()).into_string();

        assert!(html.contains("<h1>Heading</h1>"));
        assert!(html.contains("<em>text</em>"));
        assert!(!html.contains("title: "));
        assert!(html.contains("hx-get=\"/graph/notes/first\""));
        assert!(html.contains("hx-get=\"/metadata/notes/first\""));
    }
}
