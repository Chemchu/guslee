use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use chess_module::{
    ChessModule,
    types::{Game, GameRatingHistory, LichessUser, Winner},
};
use maud::{PreEscaped, html};
use serde_json::json;
use std::{collections::HashMap, time::Duration};

use crate::controllers::{AppState, wrap_content_into_full_page};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(chess_stats_by_game)
        .service(chess_games_analysis);
}

#[cached(
    time = 3600,
    key = "String",
    convert = r##"{
        format!(
            "chess-{}-is_htmx_req-{}",
            path.clone(),
            req.headers().get("HX-Request").is_some()
        )
    }"##
)]
#[get("/chess/stats/{game_mode}")]
async fn chess_stats_by_game(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> Html {
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

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(render_chess_page(game_mode, rating_history, player_stats))
    } else {
        Html::new(wrap_content_into_full_page(
            &app_state.app_name,
            render_chess_page(game_mode, rating_history, player_stats)
                .0
                .as_str(),
        ))
    }
}

#[cached(
    time = 3600,
    key = "String",
    convert = r#"{ "chess-games-analysis".to_string() }"#
)]
#[get("/chess/games/analysis")]
async fn chess_games_analysis(app_state: web::Data<AppState>) -> Html {
    let data = ChessModule::get_last_games_analysis(
        &app_state.lichess_state.lichess_token,
        &app_state.lichess_state.lichess_username,
    )
    .await;

    let games = match data {
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

    Html::new(render_analysis_page(games))
}

fn render_chess_page(
    game_mode: String,
    rating_history: Vec<GameRatingHistory>,
    player_stats: LichessUser,
) -> PreEscaped<String> {
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

    let perf_stat = match game_mode_normalized {
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

    html! {
        div class="flex flex-col w-full gap-6 md:p-6 lg:p-8 overflow-y-auto h-full"
        {
            h1 class="text-4xl font-bold"
            {
                "Chess Journey"
            }
            div id="chess-stats"
            {
                div class="text-bright-color w-full" {
                    div class="container mx-auto max-w-6xl h-full" {
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
                                    p class="text-2xl font-bold" {
                                        (format!("{:.2}%", win_rate))
                                    }
                                }
                                div {
                                    p class="text-gray-400 text-sm" { "Draw Rate" }
                                    p class="text-2xl font-bold" {
                                        (format!("{:.2}%", draw_rate))
                                    }
                                }
                                div {
                                    p class="text-gray-400 text-sm" { "Loss Rate" }
                                    p class="text-2xl font-bold" {
                                        (format!("{:.2}%", loss_rate))
                                    }
                                }
                            }
                        }
                        div {
                            div id="elo-chart" dataset=(data_json) {}
                        }
                        div
                        id="chess-games-analysis"
                        hx-get="/chess/games/analysis"
                        hx-trigger="load"
                        hx-target="this"
                        hx-swap="innerHTML"
                        class="w-full min-w-0 overflow-y-auto"
                        {
                            "Loading..."
                        }
                    }
                }
            }
        }
    }
}

fn render_analysis_page(games: Vec<Game>) -> PreEscaped<String> {
    let total = games.len();

    let mut white_openings: HashMap<String, usize> = HashMap::new();
    let mut black_openings: HashMap<String, usize> = HashMap::new();

    let mut total_accuracy_white = 0i32;
    let mut accuracy_white_count = 0i32;
    let mut total_accuracy_black = 0i32;
    let mut accuracy_black_count = 0i32;

    for game in &games {
        if let Some(opening) = &game.opening {
            *white_openings.entry(opening.name.clone()).or_insert(0) += 1;
            *black_openings.entry(opening.name.clone()).or_insert(0) += 1;
        }

        if let Some(analysis) = &game.players.white.analysis
            && let Some(acc) = analysis.accuracy
        {
            total_accuracy_white += acc;
            accuracy_white_count += 1;
        }

        if let Some(analysis) = &game.players.black.analysis
            && let Some(acc) = analysis.accuracy
        {
            total_accuracy_black += acc;
            accuracy_black_count += 1;
        }
    }

    let top_white_opening = white_openings
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, v)| (k.clone(), *v));

    let top_black_opening = black_openings
        .iter()
        .max_by_key(|(_, v)| *v)
        .map(|(k, v)| (k.clone(), *v));

    let avg_acc_white = if accuracy_white_count > 0 {
        total_accuracy_white / accuracy_white_count
    } else {
        0
    };
    let avg_acc_black = if accuracy_black_count > 0 {
        total_accuracy_black / accuracy_black_count
    } else {
        0
    };

    html! {
        div class="max-w-4xl mx-auto" {

            header {
                h1 class="text-3xl font-bold tracking-tight mb-2" { "Game Analysis" }
                div class="w-12 h-px mx-auto" {}
                p class="text-gray-400 text-xs uppercase tracking-widest" { (total) " games reviewed" }
            }

            div class="flex items-center gap-3 mb-4 pt-10" {
                p class="text-primary-color text-sm font-semibold uppercase tracking-wider" { "Favourite Openings" }
            }
            div class="grid grid-cols-1 sm:grid-cols-2 gap-px" {
                div class="bg-zinc-900 p-7 border-l-2 border-zinc-400 hover:bg-zinc-800 transition-colors" {
                    p class="text-gray-400 text-xs uppercase tracking-widest mb-3 flex items-center gap-2" {
                        span class="inline-block w-2 h-2 rounded-full bg-zinc-300" {}
                        "As White"
                    }
                    @if let Some((name, count)) = &top_white_opening {
                        p class="text-zinc-100 text-lg font-semibold leading-snug mb-2" { (name) }
                        p class="text-zinc-500 text-xs" {
                            "Played " span class="text-primary-color" { (count) } " times"
                        }
                    } @else {
                        p class="text-zinc-600 text-base" { "No data" }
                    }
                }
                div class="bg-zinc-900 p-7 border-l-2 border-primary-color hover:bg-zinc-800 transition-colors" {
                    p class="text-gray-400 text-xs uppercase tracking-widest mb-3 flex items-center gap-2" {
                        span class="inline-block w-2 h-2 rounded-full bg-primary-color" {}
                        "As Black"
                    }
                    @if let Some((name, count)) = &top_black_opening {
                        p class="text-zinc-100 text-lg font-semibold leading-snug mb-2" { (name) }
                        p class="text-zinc-500 text-xs" {
                            "Played " span class="text-primary-color" { (count) } " times"
                        }
                    } @else {
                        p class="text-zinc-600 text-base" { "No data" }
                    }
                }
            }

            @if accuracy_white_count > 0 || accuracy_black_count > 0 {
                div class="flex items-center gap-3 mb-4 mt-10" {
                    p class="text-amber-400 text-sm font-semibold uppercase tracking-wider" { "Average Accuracy" }
                    div class="flex-1 h-px bg-zinc-800" {}
                }
                div class="grid grid-cols-1 sm:grid-cols-2 gap-px bg-zinc-800 mb-10" {
                    div class="bg-zinc-900 p-6 flex items-center justify-between hover:bg-zinc-800 transition-colors" {
                        div {
                            p class="text-zinc-500 text-xs uppercase tracking-wider mb-3" { "♙ White" }
                            div class="w-32 h-1 bg-zinc-800 rounded-full overflow-hidden" {
                                div class="h-full bg-amber-400 rounded-full"
                                    style=(format!("width:{}%", avg_acc_white)) {}
                            }
                        }
                        p class="text-4xl font-bold text-amber-400" { (avg_acc_white) "%" }
                    }
                    div class="bg-zinc-900 p-6 flex items-center justify-between hover:bg-zinc-800 transition-colors" {
                        div {
                            p class="text-zinc-500 text-xs uppercase tracking-wider mb-3" { "♟ Black" }
                            div class="w-32 h-1 bg-zinc-800 rounded-full overflow-hidden" {
                                div class="h-full bg-amber-400 rounded-full"
                                    style=(format!("width:{}%", avg_acc_black)) {}
                            }
                        }
                        p class="text-4xl font-bold text-amber-400" { (avg_acc_black) "%" }
                    }
                }
            }

            div class="flex items-center gap-3 mb-4 pt-4" {
                p class="text-primary-color text-sm font-semibold uppercase tracking-wider" { "Recent Games" }
            }
            div class="flex flex-col gap-px bg-zinc-800" {
                @for game in games.iter().take(15) {
                    @let main_player_color = if game.players.white.user.as_ref().map(|u| u.name.as_str()) == Some("chemchu") { "white" } else { "black" };
                    @let (border, badge_cls, result_label) = match (&game.winner, main_player_color) {
                        (Some(Winner::White), "white") | (Some(Winner::Black), "black") => (
                            "border-l-2 border-emerald-500",
                            "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20",
                            "Win"
                        ),
                        (Some(Winner::White), "black") | (Some(Winner::Black), "white") => (
                            "border-l-2 border-rose-500",
                            "bg-rose-500/10 text-rose-400 border border-rose-500/20",
                            "Loss"
                        ),
                        (None, _) => (
                            "border-l-2 border-primary-color",
                            "bg-primary-color/10 text-primary-color border border-primary-color/20",
                            "Draw"
                        ),
                        _ => (
                            "border-l-2 border-zinc-600",
                            "bg-zinc-800 text-zinc-400 border border-zinc-600/20",
                            "Unknown"
                        ),
                    };
                    @let opening_name = game.opening.as_ref()
                        .map(|o| o.name.as_str())
                        .unwrap_or("Unknown Opening");
                    @let speed = format!("{:?}", game.speed);
                    div class=(format!("bg-zinc-900 {} px-5 py-4 flex items-center justify-between hover:bg-zinc-800 transition-all hover:pl-7 text-sm", border)) {
                        div {
                            p class="text-zinc-100" { (opening_name) }
                            p class="text-zinc-600 text-xs mt-1" { "#" (game.id) " played as " (main_player_color) }
                        }
                        div class="flex items-center gap-4" {
                            p class="text-zinc-500 text-xs uppercase tracking-wide hidden sm:block" { (speed) }
                            span class=(format!("text-xs uppercase tracking-wider px-2 py-1 rounded {}", badge_cls)) {
                                (result_label)
                            }
                        }
                    }
                }
            }
        }
    }
}
