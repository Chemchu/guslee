use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use markdown::{Constructs, Options, ParseOptions};
use maud::html;
use search_engine::types::{DEFAULT_SEARCH_LIMIT, MatchingFile, Params};
use std::collections::{HashMap, HashSet};
use std::{fs, io};

use crate::routes::{AppState, wrap_content_into_full_page};

#[get("/{post:.*}")]
pub async fn get_post(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    route: web::Path<String>,
) -> Html {
    let content: io::Result<String> = fs::read_to_string(format!("./garden/{}.md", route));

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

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(match content {
            Ok(md) => markdown::to_html_with_options(&md, &frontmatter).unwrap(),
            Err(_err) => String::from("Fallback page"),
        })
    } else {
        Html::new(match content {
            Ok(md) => wrap_content_into_full_page(
                &app_state.app_name,
                &markdown::to_html_with_options(&md, &frontmatter).unwrap(),
            ),
            Err(_err) => String::from("Fallback page"),
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
            true => {
                app_state
                    .search_engine
                    .query_from_list(vec![
                        "welcome.md",
                        "hello.md",
                        "garden_styling.md",
                        "kilbarrack.md",
                        "first_job_in_ireland.md",
                        "rathmines.md",
                    ])
                    .await
                    .matching_files
            }
            false => {
                app_state
                    .search_engine
                    .query_posts(&params.clone())
                    .await
                    .matching_files
            }
        };

        build_posts_list(matching_posts)
    } else {
        Html::new(String::from("Only HTMX requests for search engine"))
    }
}

fn build_posts_list(matching_posts: Vec<MatchingFile>) -> Html {
    let mut posts_per_topic: HashMap<String, Vec<MatchingFile>> = HashMap::default();
    let mut posts_by_filename: HashMap<String, MatchingFile> = HashMap::default();
    for m_post in matching_posts.clone() {
        posts_by_filename.insert(m_post.file_name().to_string(), m_post.clone());
        if let Some(topic) = m_post.topic() {
            posts_per_topic
                .entry(topic.clone())
                .or_default()
                .push(m_post);
        }
    }

    let mut topics_to_render: Vec<(String, Vec<MatchingFile>)> = Vec::new();
    let mut posts_to_render: Vec<MatchingFile> = Vec::new();
    let mut rendered_topics: HashSet<String> = HashSet::new();

    for matching_post in matching_posts {
        if let Some(p) = posts_by_filename.get(matching_post.file_name()) {
            if let Some(topic) = p.topic() {
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
                                        "/{}",
                                        topic_post
                                            .file_path()
                                            .strip_suffix(".md")
                                            .unwrap_or(topic_post.file_path())
                                    ))
                                    hx-target="#content-section"
                                    hx-swap="innerHTML transition:true"
                                    class="pl-3 cursor-pointer hover:text-primary-color"
                                    {
                                        (topic_post.title())
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
                        "/{}",
                        p
                            .file_path()
                            .strip_suffix(".md")
                            .unwrap_or(p.file_path())
                    ))
                    hx-target="#content-section"
                    hx-swap="innerHTML transition:true"
                    class="cursor-pointer hover:text-primary-color"
                    {
                        (p.title())
                    }
                }
            }
        }
    };
    Html::new(html)
}
