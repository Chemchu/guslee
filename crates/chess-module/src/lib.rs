use serde_json::Deserializer;
use tokio::try_join;

use crate::types::{AllGamesRatingHistory, Game, LichessUser};
pub mod types;

pub const LICHESS_API_BASE_URL: &str = "https://lichess.org";

pub struct LichessState {
    pub lichess_token: String,
    pub lichess_username: String,
    pub api_base_url: String,
}

impl LichessState {
    pub fn new(lichess_token: String, lichess_username: String) -> Self {
        Self {
            lichess_token,
            lichess_username,
            api_base_url: LICHESS_API_BASE_URL.to_string(),
        }
    }

    pub fn with_api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = api_base_url.into();
        self
    }
}

pub struct ChessModule;

impl ChessModule {
    async fn get_player_rating_history(
        state: &LichessState,
    ) -> Result<AllGamesRatingHistory, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/api/user/{}/rating-history",
                state.api_base_url, state.lichess_username
            ))
            .send()
            .await?
            .error_for_status()?;

        response.json::<AllGamesRatingHistory>().await
    }

    async fn get_player_profile(state: &LichessState) -> Result<LichessUser, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/account", state.api_base_url))
            .header("Authorization", format!("Bearer {}", state.lichess_token))
            .send()
            .await?
            .error_for_status()?;

        response.json::<LichessUser>().await
    }

    pub async fn get_last_games_analysis(
        state: &LichessState,
        number_of_games: usize,
    ) -> Result<Vec<Game>, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/api/games/user/{}?opening=true&perfType=blitz&accuracy=true&max={}",
                state.api_base_url, state.lichess_username, number_of_games
            ))
            .header("Accept", "application/x-ndjson")
            .header("Authorization", format!("Bearer {}", state.lichess_token))
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        Ok(parse_games_ndjson(&body))
    }

    pub async fn get_player_data(
        state: &LichessState,
    ) -> Result<(LichessUser, AllGamesRatingHistory), reqwest::Error> {
        let profile_future = ChessModule::get_player_profile(state);
        let rating_history_future = ChessModule::get_player_rating_history(state);

        let (profile, rating_history) = try_join!(profile_future, rating_history_future)?;

        Ok((profile, rating_history))
    }
}

fn parse_games_ndjson(body: &str) -> Vec<Game> {
    Deserializer::from_str(body)
        .into_iter::<Game>()
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GAME_LINE: &str = r#"{"id":"abc123","rated":true,"variant":"standard","speed":"rapid","perf":"rapid","createdAt":1,"lastMoveAt":2,"status":"mate","players":{"white":{"rating":1500},"black":{"rating":1480}},"winner":"white","opening":{"eco":"C50","name":"Italian Game: Giuoco Piano","ply":7}}"#;

    #[test]
    fn lichess_state_defaults_to_the_public_api() {
        let state = LichessState::new("token".into(), "user".into());

        assert_eq!(state.api_base_url, LICHESS_API_BASE_URL);
        assert_eq!(
            state
                .with_api_base_url("http://localhost:1234")
                .api_base_url,
            "http://localhost:1234"
        );
    }

    #[test]
    fn parses_every_game_in_an_ndjson_stream() {
        let body = format!("{GAME_LINE}\n{}\n", GAME_LINE.replace("abc123", "def456"));

        let games = parse_games_ndjson(&body);

        assert_eq!(games.len(), 2);
        assert_eq!(games[0].id, "abc123");
        assert_eq!(games[1].id, "def456");
        assert_eq!(
            games[0].opening.as_ref().unwrap().name,
            "Italian Game: Giuoco Piano"
        );
    }

    #[test]
    fn empty_or_invalid_ndjson_yields_no_games() {
        assert!(parse_games_ndjson("").is_empty());
        assert!(parse_games_ndjson("not json").is_empty());
    }
}
