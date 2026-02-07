use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyUser {
    pub display_name: String,
    pub id: String,
    pub images: Vec<SpotifyImage>,
    pub followers: SpotifyFollowers,
    pub external_urls: SpotifyExternalUrls,
    pub product: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyFollowers {
    pub total: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyImage {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TopTracksResponse {
    pub items: Vec<SpotifyTrack>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopArtistsResponse {
    pub items: Vec<TopArtistResponse>,
    pub total: u32,
    pub limit: u32,
    pub offset: u32,
    pub href: String,
    pub previous: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopArtistResponse {
    pub external_urls: ExternalUrls,
    pub followers: Followers,
    pub genres: Vec<String>,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub popularity: u32,
    #[serde(rename = "type")]
    pub artist_type: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub url: String,
    pub height: u32,
    pub width: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyTrack {
    pub name: String,
    pub id: String,
    pub artists: Vec<SpotifyArtist>,
    pub album: SpotifyAlbum,
    pub external_urls: SpotifyExternalUrls,
    pub preview_url: Option<String>,
    pub duration_ms: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyArtist {
    pub name: String,
    pub id: String,
    pub external_urls: SpotifyExternalUrls,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyAlbum {
    pub name: String,
    pub images: Vec<SpotifyImage>,
    pub external_urls: SpotifyExternalUrls,
}

pub struct SpotifySession {
    pub auth_token: String,
    pub refresh_token: String,
}

pub struct SpotifyState {
    pub spotify_user_id: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
    pub spotify_session: SpotifySession,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpotifyRefreshResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
    // Note: refresh_token is optional here - Spotify sometimes returns a new one
    pub refresh_token: Option<String>,
}

impl SpotifyState {
    pub async fn new(
        spotify_user_id: String,
        spotify_client_id: String,
        spotify_client_secret: String,
        auth_code: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_state = SpotifyState {
            spotify_user_id: spotify_user_id.clone(),
            spotify_client_id: spotify_client_id.clone(),
            spotify_client_secret: spotify_client_secret.clone(),
            spotify_session: SpotifySession {
                auth_token: String::new(),
                refresh_token: String::new(),
            },
        };

        let token_response = temp_state.get_initial_tokens(&auth_code).await?;
        Ok(SpotifyState {
            spotify_user_id,
            spotify_client_id,
            spotify_client_secret,
            spotify_session: SpotifySession {
                auth_token: token_response.access_token,
                refresh_token: token_response.refresh_token,
            },
        })
    }

    async fn get_initial_tokens(&self, auth_code: &str) -> Result<TokenResponse, reqwest::Error> {
        let body = format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
            auth_code, "https://localhost:8888", self.spotify_client_id, self.spotify_client_secret
        );

        let client = reqwest::Client::new();
        let response = client
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        response.json::<TokenResponse>().await
    }

    pub fn from_refresh_token(
        spotify_user_id: String,
        spotify_client_id: String,
        spotify_client_secret: String,
        refresh_token: String,
    ) -> Self {
        SpotifyState {
            spotify_user_id,
            spotify_client_id,
            spotify_client_secret,
            spotify_session: SpotifySession {
                auth_token: String::new(),
                refresh_token,
            },
        }
    }

    async fn refresh_spotify_token(&self) -> Result<(String, i64), reqwest::Error> {
        let body = format!(
            "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
            self.spotify_session.refresh_token, self.spotify_client_id, self.spotify_client_secret
        );

        let client = reqwest::Client::new();
        let response = client
            .post("https://accounts.spotify.com/api/token")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?;

        let content = response.json::<SpotifyRefreshResponse>().await?;
        Ok((
            content.access_token,
            Utc::now().timestamp() + content.expires_in,
        ))
    }

    async fn get_spotify_session(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        // Maybe track token expiry tracking here
        let (new_token, _expires_at) = self.refresh_spotify_token().await?;
        self.spotify_session.auth_token = new_token.clone();
        Ok(new_token)
    }

    pub async fn fetch_user_profile(&mut self) -> Result<SpotifyUser, Box<dyn std::error::Error>> {
        let url = "https://api.spotify.com/v1/me".to_string();
        let session = self.get_spotify_session().await?;
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", session))
            .send()
            .await?;
        let content = response.json::<SpotifyUser>().await?;
        Ok(content)
    }

    /* pub async fn fetch_top_tracks(
        &mut self,
        time_range: &str,
    ) -> Result<TopTracksResponse, Box<dyn std::error::Error>> {
        self.fetch_top_items("tracks", time_range).await
    }

    pub async fn fetch_top_artists(
        &mut self,
        time_range: &str,
    ) -> Result<TopArtistsResponse, Box<dyn std::error::Error>> {
        self.fetch_top_items("artists", time_range).await
    } */

    pub async fn fetch_top_items<T: serde::de::DeserializeOwned>(
        &mut self,
        item_type: &str,
        time_range: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.spotify.com/v1/me/top/{}?limit=5&time_range={}",
            item_type, time_range
        );
        let session = self.get_spotify_session().await?;
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", session))
            .send()
            .await?;
        let text = response.text().await?;
        let content: T = serde_json::from_str(&text)?;
        Ok(content)
    }
}
