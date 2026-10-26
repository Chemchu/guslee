use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::cached;
use chess_module::{
    ChessModule,
    types::{Game, GameRatingHistory, LichessUser, PerfStat, Perfs, Winner},
};
use maud::{PreEscaped, html};
use serde_json::json;
use std::collections::HashMap;

use crate::controllers::{AppState, render_page};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(chess_stats_by_game)
        .service(chess_games_analysis);
}

#[get("/chess/stats/{game_mode}")]
async fn chess_stats_by_game(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    req: HttpRequest,
) -> Html {
    let content = chess_stats_content(app_state.clone(), path.into_inner())
        .await
        .unwrap_or_else(|fallback| fallback);

    render_page(&app_state, &req, &content)
}

#[get("/chess/games/analysis")]
async fn chess_games_analysis(app_state: web::Data<AppState>) -> Html {
    Html::new(
        chess_games_analysis_content(app_state)
            .await
            .unwrap_or_else(|fallback| fallback),
    )
}

// Only successful responses are cached, so a Lichess outage is retried on the next request
#[cached(
    ttl_secs = 3600,
    key = "String",
    convert = r#"{ format!("chess-{}", game_mode) }"#
)]
async fn chess_stats_content(
    app_state: web::Data<AppState>,
    game_mode: String,
) -> Result<String, String> {
    let (player_stats, rating_history) = ChessModule::get_player_data(&app_state.lichess_state)
        .await
        .map_err(|err| {
            log::error!("Error fetching chess data: {}", err);
            chess_fallback_html()
        })?;

    Ok(render_chess_page(game_mode, rating_history, player_stats).into_string())
}

#[cached(
    ttl_secs = 3600,
    key = "String",
    convert = r#"{ "chess-games-analysis".to_string() }"#
)]
async fn chess_games_analysis_content(app_state: web::Data<AppState>) -> Result<String, String> {
    let games = ChessModule::get_last_games_analysis(&app_state.lichess_state, 10)
        .await
        .map_err(|err| {
            log::error!("Error fetching chess games analysis: {}", err);
            chess_fallback_html()
        })?;

    Ok(render_analysis_page(games, &app_state.lichess_state.lichess_username).into_string())
}

fn chess_fallback_html() -> String {
    let template_path =
        std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
    let fallback_md = std::fs::read_to_string(format!("{}/chess_stats_fallback.md", template_path))
        .unwrap_or_else(|_| "Error loading chess stats".to_string());
    markdown::to_html(&fallback_md)
}

// Maps the Lichess perf key used in the URL (e.g. "kingOfTheHill") to the display name
// Lichess uses in the rating history (e.g. "King of the Hill")
fn normalize_game_mode(game_mode: &str) -> String {
    match game_mode {
        "kingOfTheHill" => "King of the Hill".to_string(),
        "racingKings" => "Racing Kings".to_string(),
        "threeCheck" => "Three-check".to_string(),
        "ultraBullet" => "UltraBullet".to_string(),
        mode => {
            let mut chars = mode.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
            }
        }
    }
}

// Picks the stats for a Lichess perf key as used in the URL (e.g. "kingOfTheHill"),
// defaulting to blitz for unknown modes
fn perf_for_mode<'a>(perfs: Option<&'a Perfs>, game_mode: &str) -> Option<&'a PerfStat> {
    let perfs = perfs?;
    match game_mode {
        "ultraBullet" => perfs.ultra_bullet.as_ref(),
        "bullet" => perfs.bullet.as_ref(),
        "blitz" => perfs.blitz.as_ref(),
        "rapid" => perfs.rapid.as_ref(),
        "classical" => perfs.classical.as_ref(),
        "correspondence" => perfs.correspondence.as_ref(),
        "chess960" => perfs.chess960.as_ref(),
        "crazyhouse" => perfs.crazyhouse.as_ref(),
        "antichess" => perfs.antichess.as_ref(),
        "atomic" => perfs.atomic.as_ref(),
        "horde" => perfs.horde.as_ref(),
        "kingOfTheHill" => perfs.king_of_the_hill.as_ref(),
        "racingKings" => perfs.racing_kings.as_ref(),
        "threeCheck" => perfs.three_check.as_ref(),
        _ => perfs.blitz.as_ref(),
    }
}

fn render_chess_page(
    game_mode: String,
    rating_history: Vec<GameRatingHistory>,
    player_stats: LichessUser,
) -> PreEscaped<String> {
    let game_mode_normalized = normalize_game_mode(&game_mode);
    let game_mode_normalized = game_mode_normalized.as_str();

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

    let perf_stat = perf_for_mode(player_stats.perfs.as_ref(), &game_mode);

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
        div class="flex flex-col w-full gap-6 md:p-6 lg:p-8 overflow-y-auto h-full" {
            h1 class="text-4xl font-bold pb-4" { "Chess Journey" }
            div id="chess-stats" {
                div class="text-bright-color w-full" {
                    div class="container mx-auto max-w-6xl h-full" {

                        div class="grid grid-cols-2 md:grid-cols-4 mb-6" {
                            div class="p-5 border border-shade-color" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest mb-1" { "Current Rating" }
                                p class="text-3xl font-bold" { (current_rating) }
                            }
                            div class="p-5 border border-shade-color" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest mb-1" { "Peak Rating" }
                                p class="text-3xl font-bold text-primary-color" { (peak_rating) }
                            }
                            div class="p-5 border border-shade-color" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest mb-1" { "Games (" (game_mode) ")" }
                                p class="text-3xl font-bold" { (total_games) }
                            }
                            div class="p-5 border border-shade-color" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest mb-1" { "Total Games" }
                                p class="text-3xl font-bold" { (total_all_games) }
                            }
                        }

                        div class="grid grid-cols-3 mb-6" {
                            div class="p-5 border border-shade-color" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest mb-2" { "Win Rate" }
                                div class="w-full h-1 bg-shade-color rounded-full overflow-hidden mb-2" {
                                    div class="h-full bg-emerald-500 rounded-full"
                                        style=(format!("width:{}%", win_rate)) {}
                                }
                                p class="text-2xl font-bold text-emerald-400" { (format!("{:.2}%", win_rate)) }
                                p class="text-zinc-600 text-xs mt-1" { (win_count) " games" }
                            }
                            div class="p-5 border border-shade-color" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest mb-2" { "Draw Rate" }
                                div class="w-full h-1 bg-shade-color rounded-full overflow-hidden mb-2" {
                                    div class="h-full bg-primary-color rounded-full"
                                        style=(format!("width:{}%", draw_rate)) {}
                                }
                                p class="text-2xl font-bold text-primary-color" { (format!("{:.2}%", draw_rate)) }
                                p class="text-zinc-600 text-xs mt-1" { (draw_count) " games" }
                            }
                            div class="p-5 border border-shade-color" {
                                p class="text-zinc-500 text-xs uppercase tracking-widest mb-2" { "Loss Rate" }
                                div class="w-full h-1 bg-shade-color rounded-full overflow-hidden mb-2" {
                                    div class="h-full bg-rose-500 rounded-full"
                                        style=(format!("width:{}%", loss_rate)) {}
                                }
                                p class="text-2xl font-bold text-rose-400" { (format!("{:.2}%", loss_rate)) }
                                p class="text-zinc-600 text-xs mt-1" { (loss_count) " games" }
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
                        class="w-full min-w-0 overflow-y-auto pt-10"
                        { "Loading..." }
                    }
                }
            }
        }
    }
}

// Lichess usernames are case-insensitive
fn plays_white(game: &Game, username: &str) -> bool {
    game.players
        .white
        .user
        .as_ref()
        .is_some_and(|user| user.name.eq_ignore_ascii_case(username))
}

fn render_analysis_page(games: Vec<Game>, username: &str) -> PreEscaped<String> {
    let total = games.len();

    let mut white_openings: HashMap<String, usize> = HashMap::new();
    let mut black_openings: HashMap<String, usize> = HashMap::new();

    for game in &games {
        if let Some(opening) = &game.opening {
            if plays_white(game, username) {
                *white_openings
                    .entry(
                        opening
                            .name
                            .clone()
                            .split_once(":")
                            .unwrap_or((opening.name.clone().as_str(), ""))
                            .0
                            .to_string(),
                    )
                    .or_insert(0) += 1;
            } else {
                *black_openings
                    .entry(
                        opening
                            .name
                            .clone()
                            .split_once(":")
                            .unwrap_or((opening.name.clone().as_str(), ""))
                            .0
                            .to_string(),
                    )
                    .or_insert(0) += 1;
            }
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

    html! {
        div class="w-full mx-auto" {

            header {
                h1 class="text-3xl font-bold tracking-tight mb-2" { "Game Analysis" }
                div class="w-12 h-px mx-auto" {}
                p class="text-gray-400 text-xs uppercase tracking-widest" { (total) " games reviewed" }
            }

            div class="flex items-center gap-3 mb-4 pt-10" {
                p class="text-primary-color text-sm font-semibold uppercase tracking-wider" { "Recently played openings" }
            }
            div class="grid grid-cols-1 sm:grid-cols-2" {
                div class="p-7 border border-shade-color border-l-2 border-l-zinc-400" {
                    p class="text-gray-400 text-xs uppercase tracking-widest mb-3 flex items-center gap-2" {
                        span class="inline-block w-2 h-2 rounded-full bg-zinc-300" {}
                        "As White"
                    }
                    @if let Some((name, count)) = &top_white_opening {
                        p class="text-zinc-100 text-lg font-semibold leading-snug mb-2" { (name) }
                        p class="text-zinc-500 text-xs" {
                            "Played " span class="text-primary-color" { (count) }
                            (if *count == 1 {" time"} else {" times"}) " recently"
                        }
                    } @else {
                        p class="text-zinc-600 text-base" { "No data" }
                    }
                }
                div class="p-7 border border-shade-color border-l-2 border-l-primary-color" {
                    p class="text-gray-400 text-xs uppercase tracking-widest mb-3 flex items-center gap-2" {
                        span class="inline-block w-2 h-2 rounded-full bg-primary-color" {}
                        "As Black"
                    }
                    @if let Some((name, count)) = &top_black_opening {
                        p class="text-zinc-100 text-lg font-semibold leading-snug mb-2" { (name) }
                        p class="text-zinc-500 text-xs" {
                            "Played " span class="text-primary-color" { (count) }
                            (if *count == 1 {" time"} else {" times"}) " recently"
                        }
                    } @else {
                        p class="text-zinc-600 text-base" { "No data" }
                    }
                }
            }

            div class="flex items-center gap-3 mb-4 pt-4" {
                p class="text-primary-color text-sm font-semibold uppercase tracking-wider" { "Recent Games" }
            }
            div class="flex flex-col border-t border-shade-color" {
                @for game in games.iter() {
                    @let main_player_color = if plays_white(game, username) { "white" } else { "black" };
                    @let (border, badge_cls, result_label) = match (&game.winner, main_player_color) {
                        (Some(Winner::White), "white") | (Some(Winner::Black), "black") => (
                            "border-l-2 border-l-emerald-500",
                            "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20",
                            "Win"
                        ),
                        (Some(Winner::White), "black") | (Some(Winner::Black), "white") => (
                            "border-l-2 border-l-rose-500",
                            "bg-rose-500/10 text-rose-400 border border-rose-500/20",
                            "Loss"
                        ),
                        (None, _) => (
                            "border-l-2 border-l-primary-color",
                            "bg-primary-color/10 text-primary-color border border-primary-color/20",
                            "Draw"
                        ),
                        _ => (
                            "border-l-2 border-l-shade-color",
                            "bg-shade-color text-zinc-400 border border-shade-color",
                            "Unknown"
                        ),
                    };
                    @let opening_name = game.opening.as_ref()
                        .map(|o| o.name.as_str())
                        .unwrap_or("Unknown Opening");
                    @let speed = format!("{:?}", game.speed);
                    div class=(format!("{} px-5 py-4 flex items-center justify-between border-b border-r border-shade-color hover:bg-shade-color transition-all hover:pl-7 text-sm", border)) {
                        div {
                            p class="text-zinc-100" { (opening_name) }
                            p class="text-zinc-600 text-xs mt-1" { "#" (game.id) " played as " (main_player_color) }
                        }
                        div class="flex items-center gap-4" {
                            p class="text-zinc-500 text-xs uppercase tracking-wide hidden sm:block" { (speed) }
                            span class=(format!("text-xs uppercase tracking-wider px-2 py-1 {}", badge_cls)) {
                                (result_label)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_modes_are_capitalised() {
        assert_eq!(normalize_game_mode("rapid"), "Rapid");
        assert_eq!(normalize_game_mode("blitz"), "Blitz");
        assert_eq!(normalize_game_mode("chess960"), "Chess960");
    }

    #[test]
    fn camel_case_variants_use_the_lichess_display_name() {
        assert_eq!(normalize_game_mode("kingOfTheHill"), "King of the Hill");
        assert_eq!(normalize_game_mode("racingKings"), "Racing Kings");
        assert_eq!(normalize_game_mode("threeCheck"), "Three-check");
        assert_eq!(normalize_game_mode("ultraBullet"), "UltraBullet");
    }

    fn perfs() -> Perfs {
        serde_json::from_value(serde_json::json!({
            "blitz": {"games": 1, "rating": 1000, "rd": 50, "prog": 0},
            "rapid": {"games": 1, "rating": 1100, "rd": 50, "prog": 0},
            "ultraBullet": {"games": 1, "rating": 1200, "rd": 50, "prog": 0},
            "kingOfTheHill": {"games": 1, "rating": 1300, "rd": 50, "prog": 0},
            "racingKings": {"games": 1, "rating": 1400, "rd": 50, "prog": 0},
            "threeCheck": {"games": 1, "rating": 1500, "rd": 50, "prog": 0}
        }))
        .unwrap()
    }

    #[test]
    fn perf_lookup_uses_the_url_game_mode() {
        let perfs = perfs();
        let rating = |mode| perf_for_mode(Some(&perfs), mode).map(|p| p.rating);

        assert_eq!(rating("rapid"), Some(1100));
        assert_eq!(rating("ultraBullet"), Some(1200));
        // These used to fall back to blitz because the display name was matched instead
        assert_eq!(rating("kingOfTheHill"), Some(1300));
        assert_eq!(rating("racingKings"), Some(1400));
        assert_eq!(rating("threeCheck"), Some(1500));
    }

    #[test]
    fn unknown_modes_fall_back_to_blitz_and_missing_perfs_to_none() {
        let perfs = perfs();

        assert_eq!(
            perf_for_mode(Some(&perfs), "made-up").map(|p| p.rating),
            Some(1000)
        );
        assert_eq!(perf_for_mode(Some(&perfs), "horde").map(|p| p.rating), None);
        assert!(perf_for_mode(None, "rapid").is_none());
    }

    fn game_with_white(name: &str) -> Game {
        serde_json::from_value(serde_json::json!({
            "id": "g1", "rated": true, "variant": "standard", "speed": "rapid", "perf": "rapid",
            "createdAt": 1, "lastMoveAt": 2, "status": "mate",
            "players": {
                "white": {"user": {"id": name.to_lowercase(), "name": name}},
                "black": {"user": {"id": "opponent", "name": "Opponent"}}
            },
            "opening": {"eco": "C50", "name": "Italian Game: Giuoco Piano", "ply": 7}
        }))
        .unwrap()
    }

    #[test]
    fn white_player_is_matched_case_insensitively_against_the_configured_user() {
        assert!(plays_white(&game_with_white("SomeUser"), "someuser"));
        assert!(!plays_white(&game_with_white("Opponent2"), "someuser"));
    }

    #[test]
    fn analysis_groups_openings_by_colour_for_the_configured_user() {
        let html =
            render_analysis_page(vec![game_with_white("SomeUser")], "someuser").into_string();

        // The opening family (before the colon) is shown as the top white opening
        assert!(html.contains("Italian Game"));
    }

    #[test]
    fn empty_mode_stays_empty() {
        assert_eq!(normalize_game_mode(""), "");
    }
}
