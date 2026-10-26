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

pub const SPOTIFY_ACCOUNTS_BASE_URL: &str = "https://accounts.spotify.com";
pub const SPOTIFY_API_BASE_URL: &str = "https://api.spotify.com";

pub struct SpotifyState {
    pub spotify_user_id: String,
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
    pub spotify_session: SpotifySession,
    // Overridable so tests can point the module at a mock server
    pub accounts_base_url: String,
    pub api_base_url: String,
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
            accounts_base_url: SPOTIFY_ACCOUNTS_BASE_URL.to_string(),
            api_base_url: SPOTIFY_API_BASE_URL.to_string(),
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
            accounts_base_url: temp_state.accounts_base_url,
            api_base_url: temp_state.api_base_url,
        })
    }

    async fn get_initial_tokens(&self, auth_code: &str) -> Result<TokenResponse, reqwest::Error> {
        let body = format!(
            "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&client_secret={}",
            auth_code, "https://localhost:8888", self.spotify_client_id, self.spotify_client_secret
        );

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/token", self.accounts_base_url))
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
            accounts_base_url: SPOTIFY_ACCOUNTS_BASE_URL.to_string(),
            api_base_url: SPOTIFY_API_BASE_URL.to_string(),
        }
    }

    pub fn with_base_urls(
        mut self,
        accounts_base_url: impl Into<String>,
        api_base_url: impl Into<String>,
    ) -> Self {
        self.accounts_base_url = accounts_base_url.into();
        self.api_base_url = api_base_url.into();
        self
    }

    async fn refresh_spotify_token(&self) -> Result<(String, i64), reqwest::Error> {
        let body = format!(
            "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
            self.spotify_session.refresh_token, self.spotify_client_id, self.spotify_client_secret
        );

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/api/token", self.accounts_base_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?
            .error_for_status()?;

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
        let url = format!("{}/v1/me", self.api_base_url);
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

    pub async fn fetch_top_items<T: serde::de::DeserializeOwned>(
        &mut self,
        item_type: &str,
        time_range: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/v1/me/top/{}?limit=5&time_range={}",
            self.api_base_url, item_type, time_range
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_from_refresh_token_starts_without_an_access_token() {
        let state = SpotifyState::from_refresh_token(
            "user".into(),
            "client".into(),
            "secret".into(),
            "refresh".into(),
        );

        assert!(state.spotify_session.auth_token.is_empty());
        assert_eq!(state.spotify_session.refresh_token, "refresh");
        assert_eq!(state.accounts_base_url, SPOTIFY_ACCOUNTS_BASE_URL);
        assert_eq!(state.api_base_url, SPOTIFY_API_BASE_URL);
    }

    #[test]
    fn base_urls_can_be_overridden() {
        let state = SpotifyState::from_refresh_token(
            "user".into(),
            "client".into(),
            "secret".into(),
            "refresh".into(),
        )
        .with_base_urls("http://accounts.test", "http://api.test");

        assert_eq!(state.accounts_base_url, "http://accounts.test");
        assert_eq!(state.api_base_url, "http://api.test");
    }

    #[test]
    fn refresh_response_may_omit_a_new_refresh_token() {
        let json = r#"{"access_token":"abc","token_type":"Bearer","expires_in":3600,"scope":"user-top-read"}"#;

        let response: SpotifyRefreshResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.access_token, "abc");
        assert!(response.refresh_token.is_none());
    }

    #[test]
    fn top_tracks_response_deserializes() {
        let json = r#"{
            "items": [{
                "name": "Song",
                "id": "t1",
                "artists": [{"name": "Artist", "id": "a1", "external_urls": {"spotify": "https://open.spotify.com/artist/a1"}}],
                "album": {"name": "Album", "images": [{"url": "https://i.scdn.co/x", "height": 64, "width": 64}], "external_urls": {"spotify": "https://open.spotify.com/album/b1"}},
                "external_urls": {"spotify": "https://open.spotify.com/track/t1"},
                "preview_url": null,
                "duration_ms": 180000
            }],
            "total": 1, "limit": 5, "offset": 0
        }"#;

        let response: TopTracksResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].artists[0].name, "Artist");
        assert!(response.items[0].preview_url.is_none());
    }
}
