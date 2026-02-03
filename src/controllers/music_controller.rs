use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use maud::{PreEscaped, html};
use music_module::{SpotifyUser, TopTracksResponse};
use std::time::Duration;

use crate::controllers::{AppState, wrap_content_into_full_page};

#[cached(
    time = 3600,
    key = "String",
    convert = r##"{ 
        format!(
            "music-is_htmx_req-{}",
            req.headers().get("HX-Request").is_some()
        )
    }"##
)]
#[get("/music/profile")]
pub async fn get_user_profile(app_state: web::Data<AppState>, req: HttpRequest) -> Html {
    let mut spotify_state = app_state.spotify_state.lock().await;
    let user_profile = spotify_state.fetch_user_profile().await;
    let top_tracks = spotify_state.fetch_top_n_tracks(5).await;

    let html_to_render = match user_profile.is_err() || top_tracks.is_err() {
        true => {
            html! {
                div class="flex flex-col w-full gap-10 md:p-6 lg:p-8 overflow-auto" {
                    "Error loading spotify"
                }
            }
        }
        false => render_mock_spotify_profile(user_profile.unwrap(), top_tracks.unwrap()),
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

pub fn render_mock_spotify_profile(
    user: SpotifyUser,
    top_tracks: TopTracksResponse,
) -> PreEscaped<String> {
    html! {
        div class="flex flex-col w-full gap-10 md:p-6 lg:p-8 overflow-auto" {
            div {
                div class="flex items-center gap-6" {
                    img src=(user.images.into_iter().next().unwrap().url) class="w-24 h-24 rounded-full bg-gradient-to-br from-green-400 to-green-600 flex items-center justify-center text-4xl font-bold shadow-lg" {
                    }
                    div class="flex-1" {
                        h2 class="text-3xl font-bold mb-2" { (user.display_name) }
                        p class="text-gray-300 mb-3" { (title_case(&format!("{} subscriber", user.product))) }
                        div class="flex gap-6 text-sm text-gray-400" {
                            div {
                                span class="font-semibold text-white" { (user.followers.total) }
                                " Followers"
                            }
                        }
                    }
                }
            }

            div {
                h2 class="text-3xl font-bold text-green-400" { (format!("Top {} Songs", top_tracks.items.len())) }
                @for (i, song) in top_tracks.items.iter().enumerate() {
                    div class="bg-white bg-opacity-5 rounded-xl p-4 flex items-center gap-4 hover:bg-opacity-10 transition-all duration-200 hover:translate-x-1"         {
                        div class="text-2xl font-bold text-green-400 w-8" { (i + 1) }
                        img src=(song.album.images.first().unwrap().url) class="w-16 h-16 bg-gradient-to-br from-purple-500 to-pink-500 rounded-lg flex items-center justify-center text-2xl" {
                        }
                        div class="flex-1" {
                            h3 class="font-bold text-lg" { (song.name) }
                            p class="text-gray-400 text-sm" { (song.artists.first().unwrap().name ) }
                        }

                        div class="text-right" {
                            p class="font-semibold" { (ms_to_min(&song.duration_ms)) }
                        }
                    }
                }
            }

            // Footer Stats
            div class="grid grid-cols-3 gap-4" {
                div class="bg-white bg-opacity-10 backdrop-blur-lg rounded-2xl px-6 text-center" {
                    p class="text-3xl font-bold text-green-400" { "2,456" }
                    p class="text-gray-400 text-sm mt-1" { "Total Songs" }
                }
                div class="bg-white bg-opacity-10 backdrop-blur-lg rounded-2xl px-6 text-center" {
                    p class="text-3xl font-bold text-green-400" { "156h" }
                    p class="text-gray-400 text-sm mt-1" { "Listening Time" }
                }
                div class="bg-white bg-opacity-10 backdrop-blur-lg rounded-2xl px-6 text-center" {
                    p class="text-3xl font-bold text-green-400" { "342" }
                    p class="text-gray-400 text-sm mt-1" { "Artists" }
                }
            }
        }
    }
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn ms_to_min(ms: &u32) -> String {
    let seg = ms / 1000;
    let last_seg = seg % 60;
    let min = seg / 60;

    format!("{}:{}", min, last_seg)
}
