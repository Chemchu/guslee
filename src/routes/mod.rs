use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Html},
};
use chess_module::ChessModule;
use markdown::{Constructs, Options, ParseOptions};
use maud::html;
use search_engine::{SearchEngine, types::Params};
use std::{fs, io};

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
) -> impl Responder {
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

#[get("/search")]
pub async fn search_post(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    params: web::Query<Params>,
) -> impl Responder {
    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        let matching_files = app_state
            .search_engine
            .exec_query(&params.into_inner())
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
                            (matching_file.file_name())
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

#[get("/chess/stats/{game_mode}")]
pub async fn chess_stats_page(path: web::Path<String>) -> impl Responder {
    let game_mode = path.into_inner();
    let player_name = "chemchuu";
    let player_data = ChessModule::get_player_data(player_name);
    let player_stats = ChessModule::get_player_stats_by_game_mode(player_name, game_mode.as_str());

    if player_data.is_none() || player_stats.is_none() {
        println!("{}", player_data.is_none());
        println!("{}", player_stats.is_none());
        let fallback_html: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/templates/chess_stats_fallback.html"
        ));

        Html::new(fallback_html);
    };

    let html: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/templates/chess_stats.html"
    ));

    let html_stats = html! {
        div {
            "Last " (player_stats.unwrap().stats.count) " games"
        }
    };

    /* html.replace("{{APPNAME}}", player_data.unwrap())
    .replace("{{CONTENT}}", player_stats.unwrap()) */

    html_stats
}
