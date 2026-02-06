use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use maud::{PreEscaped, html};
use music_module::{SpotifyUser, TopArtistsResponse, TopTracksResponse};
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
    let top_tracks = spotify_state.fetch_top_tracks(5).await;
    let top_artists = spotify_state.fetch_top_artists(5).await;

    let html_to_render = match user_profile.is_err() || top_tracks.is_err() {
        true => {
            html! {
                div class="flex flex-col w-full gap-10 md:p-6 lg:p-8 overflow-auto" {
                    "Error loading spotify"
                }
            }
        }
        false => render_mock_spotify_profile(
            user_profile.unwrap(),
            top_tracks.unwrap(),
            top_artists.unwrap(),
        ),
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

#[get("/music/top-artists/{time_limit:.*}")]
pub async fn get_user_top_artists(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    route: web::Path<String>,
) -> Html {
    todo!()
}

pub fn render_mock_spotify_profile(
    user: SpotifyUser,
    top_tracks: TopTracksResponse,
    top_artists: TopArtistsResponse,
) -> PreEscaped<String> {
    html! {
        div class="flex flex-col w-full gap-8 p-6 lg:p-8 overflow-auto" {
            div {
                div class="flex items-center gap-6" {
                    a href=(user.external_urls.spotify) target="_blank" rel="noopener noreferrer" {
                        img src=(user.images.into_iter().next().unwrap().url)
                        class="w-28 h-28 transition-all duration-300 rounded-full ring-2 ring-bright-color hover:ring-primary-color shadow-2xl" {}
                    }
                    div class="flex-1" {
                        h2 class="transition-all duration-300 text-4xl font-bold mb-2 bg-clip-text text-bright-color hover:text-primary-color" {
                            a href=(user.external_urls.spotify) target="_blank" rel="noopener noreferrer" {
                                (user.display_name)
                            }
                        }
                        p class="text-gray-300 mb-3 text-lg" {
                            (title_case(&format!("{} subscriber", user.product)))
                        }
                        div class="flex gap-6 text-sm text-gray-400" {
                            div class="flex items-center gap-2" {
                                span class="font-bold text-xl" { (user.followers.total) }
                                span { "Followers" }
                            }
                        }
                    }
                }
            }

            div class="grid grid-cols-1 lg:grid-cols-2 gap-8" {
                div class="flex flex-col gap-4" {
                    div class="flex items-center gap-3 mb-2" {
                        div class="w-1 h-8 bg-primary-color rounded-full" {}
                        h2 class="text-2xl font-bold text-white" {
                            "Top " (top_tracks.items.len()) " Songs"
                        }
                    }
                    div class="flex flex-col gap-3" {
                        @for (i, song) in top_tracks.items.iter().enumerate() {
                            a href=(song.external_urls.spotify) target="_blank" rel="noopener noreferrer"
                                class="group bg-bright-color/5 backdrop-blur-sm p-4 flex items-center gap-4 hover:bg-bright-color/10 transition-all duration-300 hover:translate-x-2 hover:shadow-lg hover:shadow-primary-color/10 border border-shade-color hover:border-primary-color" {
                                div class="text-xl font-bold group-hover:text-primary-color w-8 text-center" {
                                    (i + 1)
                                }
                                img src=(song.album.images.first().unwrap().url)
                                    class="w-14 h-14 rounded-lg shadow-md group-hover:shadow-xl transition-shadow duration-300" {}
                                div class="flex-1 min-w-0" {
                                    h3 class="font-semibold text-base text-white truncate group-hover:text-primary-color transition-colors" {
                                        (song.name)
                                    }
                                    p class="text-gray-400 text-sm truncate" {
                                        (song.artists.first().unwrap().name)
                                    }
                                }
                                div class="text-right" {
                                    p class="font-medium text-sm text-gray-300" {
                                        (ms_to_min(&song.duration_ms))
                                    }
                                }
                            }
                        }
                    }
                }

                div class="flex flex-col gap-4" {
                    div class="flex items-center gap-3 mb-2" {
                        div class="w-1 h-8 bg-primary-color rounded-full" {}
                        h2 class="text-2xl font-bold text-white" {
                            "Top " (top_artists.items.len()) " Artists"
                        }
                    }
                    div class="flex flex-col gap-3" {
                        @for (i, artist) in top_artists.items.iter().enumerate() {
                            a href=(artist.external_urls.spotify) target="_blank" rel="noopener noreferrer"
                                class="group bg-bright-color/5 backdrop-blur-sm p-4 flex items-center gap-4 hover:bg-bright-color/10 transition-all duration-300 hover:translate-x-2 hover:shadow-lg hover:shadow-primary-color/10 border border-shade-color hover:border-primary-color" {
                                div class="text-xl font-bold group-hover:text-primary-color w-8 text-center" {
                                    (i + 1)
                                }
                                img src=(artist.images.first().unwrap().url)
                                    class="w-14 h-14 rounded-full shadow-md group-hover:shadow-xl transition-shadow duration-300 group-hover:ring-primary-color" {}
                                div class="flex-1 min-w-0" {
                                    h3 class="font-semibold text-base text-white truncate group-hover:text-primary-color transition-colors" {
                                        (artist.name)
                                    }
                                    p class="text-gray-400 text-sm truncate" {
                                        @if let Some(genre) = artist.genres.first() {
                                            (genre)
                                        } @else {
                                            "Artist"
                                        }
                                    }
                                }
                                div class="text-right" {
                                    p class="font-medium text-sm text-gray-300" {
                                        (artist.popularity) "%"
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
