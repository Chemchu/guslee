use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use chess_module::ChessModule;
use markdown::{Constructs, Options, ParseOptions};
use maud::html;
use search_engine::{
    SearchEngine,
    types::{DEFAULT_SEARCH_LIMIT, MatchingFile, Params},
};
use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use std::{fs, io};

const PLAYER_NAME: &str = "chemchuu";

pub struct AppState {
    pub app_name: String,
    pub search_engine: std::sync::Arc<SearchEngine>,
}

#[get("/")]
pub async fn landing(app_state: web::Data<AppState>) -> impl Responder {
    let content: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/garden/welcome.md"));

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

    Html::new(wrap_markdown_with_whole_page(
        &app_state.app_name,
        &markdown::to_html_with_options(content, &frontmatter).unwrap(),
    ))
}

#[get("/{post:.*}")]
pub async fn post(
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
            Ok(md) => wrap_markdown_with_whole_page(
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
            || params.query.as_ref().unwrap() == "*"
            || params.query.as_ref().unwrap().len() < 3;

        if is_empty_query {
            let default_posts = vec![
                "welcome.md",
                "hello.md",
                "garden_styling.md",
                "kilbarrack.md",
                "first_job_in_ireland.md",
                "rathmines.md",
            ];
            let matching_posts = app_state
                .search_engine
                .query_from_list(default_posts.clone())
                .await
                .matching_files;
            let mut posts_per_topic: HashMap<String, Vec<MatchingFile>> = HashMap::default();
            let mut posts_by_filename: HashMap<String, MatchingFile> = HashMap::default();

            for m_post in matching_posts {
                posts_by_filename.insert(m_post.file_name().to_string(), m_post.clone());
                if let Some(topic) = m_post.topic() {
                    posts_per_topic
                        .entry(topic.clone())
                        .or_default()
                        .push(m_post);
                }
            }

            // Pre-process to determine which topics to render
            let mut topics_to_render: Vec<(String, Vec<MatchingFile>)> = Vec::new();
            let mut posts_to_render: Vec<MatchingFile> = Vec::new();
            let mut rendered_topics: HashSet<String> = HashSet::new();

            for default_post in default_posts {
                if let Some(p) = posts_by_filename.get(default_post) {
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
                    @for (topic, topic_posts) in topics_to_render {
                        li {
                            details {
                                summary { (topic) }
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
                                            hx-swap="innerHTML"
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
                            hx-swap="innerHTML"
                            {
                                (p.title())
                            }
                        }
                    }
                }
            };
            return Html::new(html);
        }

        let matching_files = app_state
            .search_engine
            .query(&params.clone())
            .await
            .matching_files;

        let html = html! {
            ul {
                @for matching_file in matching_files {
                    li {
                        a href=(format!(
                            "/{}",
                            matching_file
                                .file_path()
                                .strip_suffix(".md")
                                .unwrap_or(matching_file.file_path())
                        ))
                        hx-target="#content-section"
                        hx-swap="innerHTML"
                        {
                            (matching_file.title())
                        }
                    }
                }
            }
        };

        Html::new(html)
    } else {
        Html::new(String::from("Only HTMX requests for search engine"))
    }
}

fn wrap_markdown_with_whole_page(app_name: &str, content: &str) -> String {
    let html: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index.html"));

    html.replace("{{APPNAME}}", app_name)
        .replace("{{CONTENT}}", content)
}

#[cached(time = 3600, key = "String", convert = r#"{ path.clone() }"#)]
#[get("/chess/stats/{game_mode}")]
pub async fn chess_stats_page(path: web::Path<String>) -> Html {
    let game_mode = path.into_inner();
    let player_data = ChessModule::get_player_data(PLAYER_NAME);
    let player_stats = ChessModule::get_player_stats_by_game_mode(PLAYER_NAME, game_mode.as_str());

    if player_data.is_none() || player_stats.is_none() {
        let fallback_html: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/chess_stats_fallback.md"
        ));

        return Html::new(markdown::to_html(fallback_html));
    };

    // TODO: finish chess page
    let _data = player_data.unwrap();
    let stats = player_stats.unwrap();

    let md: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/chess_stats.md"
    ));

    Html::new(markdown::to_html(
        md.replace(
            "{{CURRENT_RATING}}",
            stats.stats.rating_last.to_string().as_str(),
        )
        .replace(
            "{{PLAYED_GAMES_COUNT}}",
            stats.stats.count.to_string().as_str(),
        )
        .replace(
            "{{PLAYED_GAMES_COUNT_WHITE}}",
            stats.stats.white_game_count.to_string().as_str(),
        )
        .replace(
            "{{PLAYED_GAMES_COUNT_BLACK}}",
            stats.stats.black_game_count.to_string().as_str(),
        )
        .as_str(),
    ))
}
