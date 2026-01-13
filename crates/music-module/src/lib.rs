use chrono::Utc;
use serde::Deserialize;

#[derive(Deserialize)]
struct SpotifyLoginResponse {
    access_token: String,
    /*     token_type: String, */
    expires_in: i64,
}

pub struct MusicModule;

impl MusicModule {
    pub fn login_to_spotify(client_id: &str, client_secret: &str) -> Option<(String, i64)> {
        let response = std::thread::scope(|s| {
            s.spawn(|| {
                let body = format!(
                    "grant_type=client_credentials&client_id={}&client_secret={}",
                    client_id, client_secret
                );

                minreq::post("https://accounts.spotify.com/api/token")
                    .with_header("Content-Type", "application/x-www-form-urlencoded")
                    .with_body(body)
                    .send()
            })
            .join()
            .unwrap()
        });

        match response {
            Ok(r) => {
                let content = r.json::<SpotifyLoginResponse>().unwrap();
                Some((
                    content.access_token,
                    Utc::now().timestamp() + content.expires_in,
                ))
            }
            Err(_) => None,
        }
    }
}
