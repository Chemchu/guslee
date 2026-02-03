use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use chess_module::ChessModule;
use maud::html;
use serde_json::json;
use std::time::Duration;

use crate::controllers::{AppState, load_html_page, wrap_content_into_full_page};

#[cached(
    time = 3600,
    key = "String",
    convert = r##"{ 
        format!(
            "chess-is_htmx_req-{}",
            req.headers().get("HX-Request").is_some()
        )
    }"##
)]
#[get("/chess")]
pub async fn chess_page(app_state: web::Data<AppState>, req: HttpRequest) -> Html {
    let chess_html = load_html_page("chess_page");

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(chess_html)
    } else {
        Html::new(wrap_content_into_full_page(
            &app_state.app_name,
            &chess_html,
        ))
    }
}

#[cached(time = 3600, key = "String", convert = r#"{ path.clone() }"#)]
#[get("/chess/stats/{game_mode}")]
pub async fn chess_graph(app_state: web::Data<AppState>, path: web::Path<String>) -> Html {
    let game_mode = path.into_inner();
    let data = ChessModule::get_player_data(
        &app_state.lichess_state.lichess_token,
        &app_state.lichess_state.lichess_username,
    )
    .await;

    let (player_stats, rating_history) = match data {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error fetching chess data: {}", err);
            let template_path =
                std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
            let fallback_html =
                std::fs::read_to_string(format!("{}/chess_stats_fallback.md", template_path))
                    .unwrap_or_else(|_| "Error loading chess stats".to_string());
            return Html::new(markdown::to_html(&fallback_html));
        }
    };

    let game_mode_normalized = match game_mode.as_str() {
        "kingOfTheHill" => "King of the Hill",
        "racingKings" => "Racing Kings",
        "threeCheck" => "Three-check",
        "ultraBullet" => "UltraBullet",
        mode => {
            let mut chars = mode.chars();
            match chars.next() {
                None => "",
                Some(first) => &format!("{}{}", first.to_uppercase(), chars.as_str()),
            }
        }
    };

    let game_mode_history = rating_history
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(game_mode_normalized));

    let chart_data: Vec<_> = game_mode_history
        .map(|history| {
            history
                .points
                .iter()
                .map(|point| {
                    json!({
                        "timestamp": point.to_timestamp_ms(),
                        "rating": point.rating(),
                        "day": format!("{}-{:02}-{:02}", point.year(), point.month() + 1, point.day())
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let data_json = serde_json::to_string(&chart_data).unwrap_or_default();

    let perf_stat = match game_mode.as_str() {
        "Blitz" => player_stats.perfs.as_ref().and_then(|p| p.blitz.as_ref()),
        "Bullet" => player_stats.perfs.as_ref().and_then(|p| p.bullet.as_ref()),
        "Rapid" => player_stats.perfs.as_ref().and_then(|p| p.rapid.as_ref()),
        "Classical" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.classical.as_ref()),
        "Correspondence" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.correspondence.as_ref()),
        "Chess960" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.chess960.as_ref()),
        "Crazyhouse" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.crazyhouse.as_ref()),
        "Antichess" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.antichess.as_ref()),
        "Atomic" => player_stats.perfs.as_ref().and_then(|p| p.atomic.as_ref()),
        "Horde" => player_stats.perfs.as_ref().and_then(|p| p.horde.as_ref()),
        "KingOfTheHill" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.king_of_the_hill.as_ref()),
        "RacingKings" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.racing_kings.as_ref()),
        "ThreeCheck" => player_stats
            .perfs
            .as_ref()
            .and_then(|p| p.three_check.as_ref()),
        _ => player_stats.perfs.as_ref().and_then(|p| p.blitz.as_ref()), // default to blitz
    };

    let current_rating = perf_stat.map(|p| p.rating).unwrap_or(0);
    let total_games = perf_stat.map(|p| p.games).unwrap_or(0);

    let peak_rating = game_mode_history
        .and_then(|h| h.peak_rating())
        .unwrap_or(current_rating as i32);

    let count = player_stats.count.as_ref();
    let win_count = count.map(|c| c.win).unwrap_or(0);
    let draw_count = count.map(|c| c.draw).unwrap_or(0);
    let loss_count = count.map(|c| c.loss).unwrap_or(0);
    let total_all_games = count.map(|c| c.all).unwrap_or(0);

    let win_rate = if total_all_games > 0 {
        (win_count as f64 / total_all_games as f64) * 100.0
    } else {
        0.0
    };
    let draw_rate = if total_all_games > 0 {
        (draw_count as f64 / total_all_games as f64) * 100.0
    } else {
        0.0
    };
    let loss_rate = if total_all_games > 0 {
        (loss_count as f64 / total_all_games as f64) * 100.0
    } else {
        0.0
    };

    let rating_html = html! {
        div class="text-text-color w-full" {
            div class="container mx-auto max-w-6xl" {
                div {
                    div class="grid grid-cols-2 md:grid-cols-4 gap-4" {
                        div {
                            p class="text-gray-400 text-sm" { "Current Rating" }
                            p class="text-2xl font-bold" { (current_rating) }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Peak Rating" }
                            p class="text-2xl font-bold" { (peak_rating) }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Games (" (game_mode) ")" }
                            p class="text-2xl font-bold" { (total_games) }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Total Games" }
                            p class="text-2xl font-bold" { (total_all_games) }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Win Rate" }
                            p class="text-2xl font-bold text-green-500" {
                                (format!("{:.1}%", win_rate))
                            }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Draw Rate" }
                            p class="text-2xl font-bold text-yellow-500" {
                                (format!("{:.1}%", draw_rate))
                            }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Loss Rate" }
                            p class="text-2xl font-bold text-red-500" {
                                (format!("{:.1}%", loss_rate))
                            }
                        }
                    }
                }

                div {
                    div id="elo-chart" dataset=(data_json) {}
                }
            }
        }
    };

    Html::new(rating_html)
}
