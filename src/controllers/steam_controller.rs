use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use maud::{PreEscaped, html};
use std::time::Duration;

use crate::controllers::{AppState, wrap_content_into_full_page};

struct SteamProfile {
    personaname: &'static str,
    realname: Option<&'static str>,
    avatar_full: &'static str,
    profileurl: &'static str,
    steamid: &'static str,
    personastate: u32,
    timecreated: Option<u64>,
    loccountrycode: Option<&'static str>,
    level: u32,
    game_count: u32,
}

struct RecentGame {
    name: &'static str,
    appid: &'static str,
    playtime_2weeks: u32,  // in minutes
    playtime_forever: u32, // in minutes
    img_icon_url: &'static str,
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
pub async fn steam_page(app_state: web::Data<AppState>, req: HttpRequest) -> Html {
    let html_to_render = match false {
        true => {
            html! {
                div class="flex flex-col w-full gap-10 md:p-6 lg:p-8 overflow-auto" {
                    "Error loading Steam page"
                }
            }
        }
        false => render_mock_steam_profile(),
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

// Mock Steam profile data
fn render_mock_steam_profile() -> PreEscaped<String> {
    let profile = SteamProfile {
        personaname: "ExampleGamer",
        realname: Some("John Doe"),
        avatar_full: "https://avatars.steamstatic.com/fef49e7fa7e1997310d705b2a6158ff8dc1cdfeb_full.jpg",
        profileurl: "https://steamcommunity.com/id/examplegamer/",
        steamid: "76561198000000000",
        personastate: 1, // Online
        timecreated: Some(1234567890),
        loccountrycode: Some("US"),
        level: 42,
        game_count: 156,
    };

    let recent_games = vec![
        RecentGame {
            name: "Counter-Strike 2",
            appid: "730",
            playtime_2weeks: 1240,
            playtime_forever: 25680,
            img_icon_url: "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/730/69f7ebe2735c366c65c0b33dae00e12dc40edbe4.jpg",
        },
        RecentGame {
            name: "Baldur's Gate 3",
            appid: "1086940",
            playtime_2weeks: 480,
            playtime_forever: 8920,
            img_icon_url: "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/1086940/94c1280d9e28ab2950a3674c6cd9597e03d3e78f.jpg",
        },
        RecentGame {
            name: "Cyberpunk 2077",
            appid: "1091500",
            playtime_2weeks: 320,
            playtime_forever: 4560,
            img_icon_url: "https://cdn.cloudflare.steamstatic.com/steamcommunity/public/images/apps/1091500/4d5b3a950f2b92d88be84f29c2e0c82f7f74c73e.jpg",
        },
    ];

    html! {
        div class="flex flex-col w-full gap-10 md:p-6 lg:p-8 overflow-auto" {
            div class="flex flex-col gap-6" {
                h1 class="text-4xl font-bold text-white" { "Steam Profile" }

                div class="bg-gradient-to-br from-slate-800 to-slate-900 rounded-lg shadow-2xl p-6 border border-purple-700" {
                    div class="flex flex-col md:flex-row gap-6 items-start" {
                        div class="flex-shrink-0" {
                            div class="relative" {
                                img
                                    src=(profile.avatar_full)
                                    alt="Steam Avatar"
                                    class="w-32 h-32 rounded-lg border-4 border-slate-700 shadow-xl";
                                div class="absolute bottom-2 right-2 w-6 h-6 bg-blue-400 rounded-full border-4 border-slate-900 shadow-lg"
                                    title="Online" {}
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
                                    span class="px-3 py-1 bg-blue-500/20 text-blue-400 rounded-full text-sm font-semibold border border-blue-500/30" {
                                        "● Online"
                                    }
                                    span class="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-full text-sm font-semibold border border-purple-500/30" {
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
                                class="px-6 py-3 bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 text-white rounded-lg font-semibold shadow-lg hover:shadow-xl transition-all duration-200 flex items-center gap-2"
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
                div class="bg-gradient-to-br from-indigo-900/50 to-indigo-800/50 rounded-lg p-6 border border-indigo-700/50 shadow-lg" {
                    div class="flex items-center gap-3" {
                        div class="p-3 bg-indigo-500/20 rounded-lg" {
                            svg class="w-8 h-8 text-indigo-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" {}
                            }
                        }
                        div {
                            p class="text-slate-400 text-sm font-medium" { "Total Games" }
                            p class="text-3xl font-bold text-white" { (profile.game_count) }
                        }
                    }
                }

                @if let Some(created) = profile.timecreated {
                    div class="bg-gradient-to-br from-emerald-900/50 to-emerald-800/50 rounded-lg p-6 border border-emerald-700/50 shadow-lg" {
                        div class="flex items-center gap-3" {
                            div class="p-3 bg-emerald-500/20 rounded-lg" {
                                svg class="w-8 h-8 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                                    path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" {}
                                }
                            }
                            div {
                                p class="text-slate-400 text-sm font-medium" { "Member Since" }
                                p class="text-xl font-bold text-white" { "2009" }
                            }
                        }
                    }
                }

                @if let Some(country) = profile.loccountrycode {
                    div class="bg-gradient-to-br from-amber-900/50 to-amber-800/50 rounded-lg p-6 border border-amber-700/50 shadow-lg" {
                        div class="flex items-center gap-3" {
                            div class="p-3 bg-amber-500/20 rounded-lg" {
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
                    svg class="w-7 h-7 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" {
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" {}
                        path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" {}
                    }
                    "Recently Played"
                }

                div class="grid grid-cols-1 gap-4" {
                    @for game in &recent_games {
                        div class="bg-gradient-to-r from-slate-800 to-slate-900 rounded-lg p-5 border border-slate-700 hover:border-blue-500/50 transition-all duration-200 shadow-lg hover:shadow-xl" {
                            div class="flex gap-4 items-center" {
                                img
                                    src=(game.img_icon_url)
                                    alt=(game.name)
                                    class="w-16 h-16 rounded-lg border-2 border-slate-600 shadow-md flex-shrink-0";

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

                                div class="hidden md:block" {
                                    div class="w-24 h-2 bg-slate-700 rounded-full overflow-hidden" {
                                        div
                                            class="h-full bg-gradient-to-r from-blue-500 to-purple-500 rounded-full"
                                            style=(format!("width: {}%", ((game.playtime_2weeks as f32 / game.playtime_forever as f32) * 100.0).min(100.0))) {}
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
