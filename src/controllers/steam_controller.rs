use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use games_module::{RecentGame, SteamProfile, TopGame};
use maud::{PreEscaped, html};
use std::time::Duration;

use crate::controllers::{AppState, wrap_content_into_full_page};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(steam_page);
}

#[cached(
    time = 3600,
    key = "String",
    convert = r##"{ 
        format!(
            "steam-is_htmx_req-{}",
            req.headers().get("HX-Request").is_some()
        )
    }"##
)]
#[get("/steam")]
async fn steam_page(app_state: web::Data<AppState>, req: HttpRequest) -> Html {
    let profile = app_state.steam_state.get_profile().await;
    let recent_games = app_state.steam_state.get_recent_games(5).await;
    let top_games = app_state.steam_state.get_top_played_games(5).await;
    let html_to_render = match profile.is_err() || recent_games.is_err() || top_games.is_err() {
        true => {
            html! {
                div class="flex flex-col w-full gap-10 md:p-6 lg:p-8 overflow-auto" {
                    "Error loading Steam page"
                }
            }
        }
        false => render_steam_page(profile.unwrap(), recent_games.unwrap(), top_games.unwrap()),
    };

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(html_to_render)
    } else {
        Html::new(wrap_content_into_full_page(
            &app_state.app_name,
            &html_to_render.0,
        ))
    }
}

fn render_steam_page(
    profile: SteamProfile,
    recent_games: Vec<RecentGame>,
    top_games: Vec<TopGame>,
) -> PreEscaped<String> {
    html! {
        @let status_text = match profile.personastate {
            0 => "Offline",
            1 => "Online",
            2 => "Busy",
            3 => "Away",
            4 => "Snooze",
            5 => "Looking to Trade",
            6 => "Looking to Play",
            _ => "Unknown",
        };

        @let status_classes = match profile.personastate {
            0 => "px-3 py-1 bg-slate-500/20 text-slate-400 rounded-full text-sm font-semibold border border-slate-500/30",
            1 => "px-3 py-1 bg-blue-500/20 text-blue-400 rounded-full text-sm font-semibold border border-blue-500/30",
            2 => "px-3 py-1 bg-red-500/20 text-red-400 rounded-full text-sm font-semibold border border-red-500/30",
            3 => "px-3 py-1 bg-yellow-500/20 text-yellow-400 rounded-full text-sm font-semibold border border-yellow-500/30",
            4 => "px-3 py-1 bg-purple-500/20 text-purple-400 rounded-full text-sm font-semibold border border-purple-500/30",
            5 => "px-3 py-1 bg-green-500/20 text-green-400 rounded-full text-sm font-semibold border border-green-500/30",
            6 => "px-3 py-1 bg-emerald-500/20 text-emerald-400 rounded-full text-sm font-semibold border border-emerald-500/30",
            _ => "px-3 py-1 bg-slate-500/20 text-slate-400 rounded-full text-sm font-semibold border border-slate-500/30",
        };

        @let pfp_indicator_color = match profile.personastate {
            0 => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-slate-500",
            1 => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-blue-400",
            2 => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-red-500",
            3 => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-yellow-500",
            4 => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-purple-500",
            5 => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-green-500",
            6 => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-emerald-500",
            _ => "absolute bottom-2 right-2 w-6 h-6 rounded-full border-4 border-slate-900 shadow-lg bg-slate-500",
        };

        div class="flex flex-col w-full gap-10 md:p-6 lg:p-8 overflow-auto" {
            div class="flex flex-col gap-6" {
                div class="bg-gradient-to-br from-slate-800 to-slate-900 shadow-2xl p-6 border border-shade-color" {
                    div class="flex flex-col md:flex-row gap-6 items-start" {
                        div class="flex-shrink-0" {
                            div class="relative" {
                                a href=(profile.profileurl) target="_blank" rel="noopener noreferrer"
                                {
                                    img
                                        src=(profile.avatar_full)
                                        alt="Steam Avatar"
                                        class="transition-all duration-200 w-32 h-32 border-4 border-slate-700 shadow-xl hover:border-primary-color";
                                }
                                div
                                class=(pfp_indicator_color)
                                title=(status_text) {}
                            }
                        }

                        div class="flex-1 min-w-0" {
                            div class="flex flex-col gap-3" {
                                h2 class="text-3xl font-bold text-white truncate" {
                                    (profile.personaname)
                                }

                                @if let Some(realname) = profile.realname {
                                    p class="text-slate-400 text-lg" { (realname) }
                                }

                                div class="flex flex-wrap gap-2 items-center" {
                                    span class=(status_classes) {
                                        "● " (status_text)
                                    }

                                    span class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-full text-sm font-semibold border border-purple-500/30"
                                    {
                                        "Level " (profile.level)
                                    }
                                }

                                p class="text-slate-500 text-sm font-mono" {
                                    "Steam ID: " (profile.steamid)
                                }
                            }
                        }

                        div class="flex-shrink-0 self-start md:self-center" {
                            a
                                href=(profile.profileurl)
                                target="_blank"
                                class="px-6 py-3 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white font-semibold shadow-lg hover:shadow-xl transition-all duration-200 flex items-center gap-2"
                            {
                                "View on Steam"
                                svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" {}
                                }
                            }
                        }
                    }
                }
            }

            div class="grid grid-cols-1 md:grid-cols-3 gap-4" {
                div class="bg-gradient-to-br from-indigo-900/50 to-indigo-800/50 p-6 border border-shade-color shadow-lg" {
                    div class="flex items-center gap-3" {
                        div class="p-3 bg-indigo-500/20" {
                            svg class="w-8 h-8 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" {}
                            }
                        }
                        div {
                            p class="text-slate-400 text-sm font-medium" { "Total Games" }
                            p class="text-3xl font-bold text-white" { (profile.game_count - 1) }
                        }
                    }
                }

                @if let Some(_created) = profile.timecreated {
                    div class="bg-gradient-to-br from-emerald-900/50 to-emerald-800/50 p-6 border border-emerald-700/50 shadow-lg" {
                        div class="flex items-center gap-3" {
                            div class="p-3 bg-emerald-500/20" {
                                svg class="w-8 h-8 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" {}
                                }
                            }
                            div {
                                p class="text-slate-400 text-sm font-medium" { "Member Since" }
                                p class="text-xl font-bold text-white" {
                                    @if let Some(timestamp) = profile.timecreated {
                                        (chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0)
                                            .unwrap()
                                            .format("%b %Y"))
                                    } @else {
                                        "Unknown"
                                    }
                                }
                            }
                        }
                    }
                }

                @if let Some(country) = profile.loccountrycode {
                    div class="bg-gradient-to-br from-amber-900/50 to-amber-800/50 p-6 border border-amber-700/50 shadow-lg" {
                        div class="flex items-center gap-3" {
                            div class="p-3 bg-amber-500/20" {
                                svg class="w-8 h-8 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" {}
                                }
                            }
                            div {
                                p class="text-slate-400 text-sm font-medium" { "Country" }
                                p class="text-xl font-bold text-white" { (country) }
                            }
                        }
                    }
                }
            }

            div class="flex flex-col gap-4" {
                h2 class="text-2xl font-bold text-white flex items-center gap-2" {
                    svg class="w-7 h-7 text-primary-color" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" {}
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" {}
                    }
                    "Recently Played"
                }

                div class="grid grid-cols-1 gap-4" {
                    @for game in &recent_games {
                        div class="bg-gradient-to-r from-slate-800 to-slate-900 p-5 border border-slate-700 hover:border-primary-color transition-all duration-200 shadow-lg hover:shadow-xl" {
                            div class="flex gap-4 items-center" {
                                img
                                    src=(game.img_logo_url)
                                    onerror={"this.onerror=null; this.src='http://media.steampowered.com/steamcommunity/public/images/apps/"(game.appid)"/"(game.img_icon_url)".jpg';"}
                                    alt=(game.name)
                                    class="w-16 h-16 border-2 border-slate-600 shadow-md flex-shrink-0 object-cover";

                                div class="flex-1 min-w-0" {
                                    h3 class="text-xl font-bold text-white truncate mb-2" {
                                        (game.name)
                                    }
                                    div class="flex flex-wrap gap-4 text-sm" {
                                        div class="flex items-center gap-1 text-slate-400" {
                                            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" {}
                                            }
                                            span { (format_playtime(game.playtime_2weeks)) " (2 weeks)" }
                                        }
                                        div class="flex items-center gap-1 text-slate-500" {
                                            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" {}
                                            }
                                            span { (format_playtime(game.playtime_forever)) " total" }
                                        }
                                    }
                                }

                                div class="hidden md:flex md:flex-col" {
                                    div class="w-24 h-2 bg-slate-700 overflow-hidden" {
                                        div
                                            class="h-full bg-primary-color"
                                            style=(format!("width: {}%", game.achievement_progress_percentage)) {}
                                    }
                                    span class="w-full text-slate-500 text-right" {
                                        (game.unlocked_achievements) "/" (game.total_achievements)
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div class="flex flex-col gap-4" {
                h2 class="text-2xl font-bold text-white flex items-center gap-2" {
                    svg class="w-7 h-7 text-primary-color" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" {}
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" {}
                    }
                    "Top Played Games"
                }
                div class="grid grid-cols-1 gap-4" {
                    @for game in &top_games {
                        div class="bg-gradient-to-r from-slate-800 to-slate-900 p-5 border border-slate-700 hover:border-primary-color transition-all duration-200 shadow-lg hover:shadow-xl" {
                            div class="flex gap-4 items-center" {
                                img
                                    src=(game.img_logo_url)
                                    onerror={"this.onerror=null; this.src='http://media.steampowered.com/steamcommunity/public/images/apps/"(game.appid)"/"(game.img_icon_url)".jpg';"}
                                    alt=(game.name)
                                    class="w-16 h-16 border-2 border-slate-600 shadow-md flex-shrink-0 object-cover";

                                div class="flex-1 min-w-0" {
                                    h3 class="text-xl font-bold text-white truncate mb-2" {
                                        (game.name)
                                    }
                                    div class="flex flex-wrap gap-4 text-sm" {
                                        div class="flex items-center gap-1 text-slate-400" {
                                            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" {}
                                            }
                                            span { (format_playtime(game.playtime_2weeks.unwrap_or(0))) " (2 weeks)" }
                                        }
                                        div class="flex items-center gap-1 text-slate-500" {
                                            svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" {}
                                            }
                                            span { (format_playtime(game.playtime_forever)) " total" }
                                        }
                                    }
                                }

                                div class="hidden md:flex md:flex-col" {
                                    div class="w-24 h-2 bg-slate-700 overflow-hidden" {
                                        div
                                            class="h-full bg-primary-color"
                                            style=(format!("width: {}%", game.achievement_progress_percentage)) {}
                                    }
                                    span class="w-full text-slate-500 text-right" {
                                        (game.unlocked_achievements) "/" (game.total_achievements)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_playtime(minutes: u32) -> String {
    let hours = minutes / 60;
    if hours == 0 {
        format!("{}m", minutes)
    } else if hours < 100 {
        format!("{:.1}h", minutes as f32 / 60.0)
    } else {
        format!("{}h", hours)
    }
}
