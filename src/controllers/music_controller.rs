use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use cached::proc_macro::cached;
use chrono::Utc;
use std::time::Duration;

use crate::controllers::AppState;
use music_module::MusicModule;

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
#[get("/music/top_5_artists")]
pub async fn get_user_profile(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Html {
    let (token, expiring_timestamp) = &app_state.spotify_state.spotify_session;

    let now = Utc::now().timestamp();

    if now > *expiring_timestamp {
        todo!("Retrieve data")
    } else {
        let login_result = MusicModule::login_to_spotify(
            &app_state.spotify_state.spotify_client_id,
            &app_state.spotify_state.spotify_client_secret,
        );
        todo!("Store token and update timestamp")
    }

    todo!("Return HTML")
}
