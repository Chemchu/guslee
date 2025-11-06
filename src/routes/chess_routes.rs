use actix_web::{
    get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use chess_module::ChessModule;
use maud::html;
use serde_json::json;
use std::time::Duration;

use crate::routes::load_html_page;

const PLAYER_NAME: &str = "chemchuu";

#[cached(time = 3600)]
#[get("/chess")]
pub async fn chess_page() -> Html {
    load_html_page("chess_page")
}

#[cached(time = 3600, key = "String", convert = r#"{ path.clone() }"#)]
#[get("/chess/stats/{game_mode}")]
pub async fn chess_graph(path: web::Path<String>) -> Html {
    let game_mode = path.into_inner();
    let data = ChessModule::get_player_data(PLAYER_NAME);
    let stats = ChessModule::get_player_stats_by_game_mode(PLAYER_NAME, game_mode.as_str());

    if data.is_none() || stats.is_none() {
        let template_path =
            std::env::var("TEMPLATE_PATH").unwrap_or_else(|_| "./templates".to_string());
        let fallback_html =
            std::fs::read_to_string(format!("{}/chess_stats_fallback.md", template_path))
                .unwrap_or_else(|_| "Error loading chess stats".to_string());
        return Html::new(markdown::to_html(&fallback_html));
    };

    let player_stats = stats.unwrap();

    let chart_data: Vec<_> = player_stats
        .stats
        .history
        .iter()
        .map(|h| {
            let timestamp_ms = h.timestamp;
            json!({
                "timestamp": timestamp_ms,
                "rating": h.rating,
                "day": h.day
            })
        })
        .collect();

    let data_json = serde_json::to_string(&chart_data).unwrap_or_default();

    let total_games = player_stats.stats.count;
    let win_rate = if total_games > 0 {
        (player_stats.stats.win_count as f64 / total_games as f64) * 100.0
    } else {
        0.0
    };
    let draw_rate = if total_games > 0 {
        (player_stats.stats.draw_count as f64 / total_games as f64) * 100.0
    } else {
        0.0
    };
    let loss_rate = if total_games > 0 {
        (player_stats.stats.loss_count as f64 / total_games as f64) * 100.0
    } else {
        0.0
    };

    let rating_html = html! {
        div class="text-text-color w-full" {
            div class="container mx-auto max-w-6xl" {
                div {
                    div class="grid grid-cols-2 md:grid-cols-4 gap-4" {
                        div {
                            p class="text-gray-400 text-sm" { "ELO" }
                            p class="text-2xl font-bold" { (player_stats.stats.rating_last) }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Peak ELO" }
                            p class="text-2xl font-bold" { (player_stats.stats.rating_max) }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Most used white opening" }
                            p class="text-2xl font-bold text-red-500" {
                                "Italian Game"
                            }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Most used black opening" }
                            p class="text-2xl font-bold text-red-500" {
                                "Modern Defense"
                            }
                        }
                        div {
                            p class="text-gray-400 text-sm" { "Games in 90 days" }
                            p class="text-2xl font-bold" { (total_games) }
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

                div class="grid grid-cols-1 md:grid-cols-2 gap-10" {
                    div {
                        h3 class="text-lg font-semibold" { "Win/Loss Breakdown" }
                        div class="space-y-2" {
                            div class="flex justify-between" {
                                span class="text-gray-400" { "As White:" }
                                span {
                                    span class="text-green-500" { (player_stats.stats.white_win_count) }
                                    " / "
                                    span class="text-yellow-500" { (player_stats.stats.white_draw_count) }
                                    " / "
                                    span class="text-red-500" { (player_stats.stats.white_loss_count) }
                                }
                            }
                            div class="flex justify-between" {
                                span class="text-gray-400" { "As Black:" }
                                span {
                                    span class="text-green-500" { (player_stats.stats.black_win_count) }
                                    " / "
                                    span class="text-yellow-500" { (player_stats.stats.black_draw_count) }
                                    " / "
                                    span class="text-red-500" { (player_stats.stats.black_loss_count) }
                                }
                            }
                        }
                    }

                    div {
                        h3 class="text-lg font-semibold" { "Performance" }
                        div class="space-y-2" {
                            div class="flex justify-between" {
                                span class="text-gray-400" { "Avg Opponent:" }
                                span { (format!("{:.0}", player_stats.stats.opponent_rating_avg)) }
                            }
                            div class="flex justify-between" {
                                span class="text-gray-400" { "Current Streak:" }
                                span class={
                                    @if player_stats.stats.streak_last >= 0 { "text-green-500" }
                                    @else { "text-red-500" }
                                } {
                                    (player_stats.stats.streak_last.abs())
                                    @if player_stats.stats.streak_last >= 0 { " W" } @else { " L" }
                                }
                            }
                            @if player_stats.stats.accuracy_count > 0 {
                                div class="flex justify-between" {
                                    span class="text-gray-400" { "Avg Accuracy:" }
                                    span { (format!("{:.1}%", player_stats.stats.accuracy_avg)) }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    Html::new(rating_html)
}
