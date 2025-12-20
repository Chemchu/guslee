use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Data, Html},
};
use cached::proc_macro::cached;
use markdown::{Constructs, Options, ParseOptions};
use maud::{PreEscaped, html};
use search_engine::{
    types::{DEFAULT_SEARCH_LIMIT, Params},
    utils::Post,
};
use std::collections::{HashMap, HashSet};
use std::{fs, io};

use crate::controllers::{AppState, wrap_content_into_full_page};

#[get("/")]
pub async fn landing(app_state: web::Data<AppState>) -> impl Responder {
    let welcome_path = format!("{}/welcome.md", app_state.garden_path);

    let content = match std::fs::read_to_string(&welcome_path) {
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
pub async fn get_post_page(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    route: web::Path<String>,
) -> Html {
    let content: io::Result<String> = fs::read_to_string(format!("./garden/{}.md", route));

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(match content {
            Ok(md) => post_page_shell(md, route.to_string()).into_string(),
            Err(_err) => String::from("Post not found"),
        })
    } else {
        Html::new(match content {
            Ok(md) => wrap_content_into_full_page(
                &app_state.app_name,
                post_page_shell(md, route.to_string())
                    .into_string()
                    .as_str(),
            ),
            Err(_err) => String::from("Post not found"),
        })
    }
}

#[cached(
    key = "String",
    convert = r##"{ 
        format!(
            "{}:{}",
            params
                .clone()
                .query
                .clone()
                .unwrap_or("empty".to_string()),
            params
                .clone()
                .limit
                .clone()
                .unwrap_or(DEFAULT_SEARCH_LIMIT)
                .value()
        )
    }"##
)]
#[get("/search")]
pub async fn search_post(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    params: web::Query<Params>,
) -> Html {
    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        let is_empty_query = params.query.is_none()
            || params.query.as_ref().unwrap().is_empty()
            || params.query.as_ref().unwrap().len() < 3;

        let matching_posts = match is_empty_query {
            true => get_default_posts(app_state).await,
            false => app_state.search_engine.query_posts(&params.clone()).await,
        };

        build_posts_list(matching_posts)
    } else {
        Html::new(String::from("Only HTMX requests for search engine"))
    }
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
        .search_engine
        .raw_query::<Vec<Post>>(
            format!(
                "SELECT * FROM posts WHERE file_name IN [{}]",
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
        .search_engine
        .raw_query::<Vec<Post>>(
            format!(
                "SELECT * FROM posts WHERE file_name NOT IN [{}]",
                default_posts_string,
            )
            .as_str(),
        )
        .await;

    [ordered_files, all_missing_posts].concat()
}

fn build_posts_list(matching_posts: Vec<Post>) -> Html {
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
    Html::new(html)
}

#[get("/{url:.*}")]
pub async fn fallback_route() -> impl Responder {
    String::from("Fallback page")
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
        id="content-section"
        class="prose prose-theme w-full max-w-full p-4 md:p-6 lg:p-8 overflow-auto text-sm md:text-base"
            {
               (PreEscaped(markdown::to_html_with_options(&md, &frontmatter).unwrap()))
            }
        div
        id="right-section"
        class="flex flex-col min-w-[16rem] xl:min-w-[20rem] max-w-md h-full border-l border-shade-color"
        {
            div
            class="flex flex-col flex-grow w-full min-h-10 max-h-[34vh] border-b border-shade-color"
            {
                div
                id="upper-right-section"
                hx-get=(format!("/graph/{}", post_path))
                hx-trigger="load, contentUpdated from:document"
                hx-target="#upper-right-section"
                hx-swap="innerHTML"
                class="flex w-full flex-grow cursor-grab active:cursor-grabbing"
                {
                    "Loading..."
                }
            }
            div
            class="flex flex-col flex-grow w-full"
            {
                div
                id="bottom-right-section"
                hx-get=(format!("/metadata/{}", post_path))
                hx-trigger="load, contentUpdated from:document"
                hx-target="#bottom-right-section"
                hx-swap="innerHTML"
                class="flex w-full flex-grow p-2"
                {
                    "Loading..."
                }
            }
        }
    }
}
