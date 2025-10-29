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
                        "back_to_it_again.md",
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
            @for (index, (topic, topic_posts)) in topics_to_render.iter().enumerate() {
                li {
                    details
                    open[index == 0]
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
                                    hx-swap="innerHTML"
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
                    hx-swap="innerHTML"
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

fn wrap_markdown_with_whole_page(app_name: &str, content: &str) -> String {
    let html: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/index.html"));

    html.replace("{{APPNAME}}", app_name)
        .replace("{{CONTENT}}", content)
}

#[get("/graph")]
pub async fn graph_network(app_state: web::Data<AppState>, req: HttpRequest) -> Html {
    let current_url = req.headers().get("HX-Current-URL");
    let graph_data = if let Some(current_url) = current_url {
        let result: Vec<&str> = current_url.to_str().unwrap().splitn(4, '/').collect();
        let file_name = result
            .get(3)
            .filter(|s| !s.is_empty())
            .unwrap_or(&"welcome");
        let file_path = format!("{}.md", file_name);
        app_state.search_engine.get_related_posts(&file_path).await
    } else {
        search_engine::types::GraphData {
            nodes: vec![],
            edges: vec![],
        }
    };

    let nodes_json = serde_json::to_string(&graph_data.nodes).unwrap();
    let edges_json = serde_json::to_string(&graph_data.edges).unwrap();

    let graph = html! {
        div #graph-container
            style="width: 100%; height: 100%;"
            data-nodes=(nodes_json)
            data-edges=(edges_json) {}
    };

    Html::new(graph)
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
