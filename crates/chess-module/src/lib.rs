use serde_json::Deserializer;
use tokio::try_join;

use crate::types::{AllGamesRatingHistory, Game, LichessUser};
pub mod types;

pub struct LichessState {
    pub lichess_token: String,
    pub lichess_username: String,
}

pub struct ChessModule;

impl ChessModule {
    pub async fn get_player_stats_by_game_mode(token: &str) -> Result<LichessUser, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://lichess.org/api/account")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        response.json::<LichessUser>().await
    }

    async fn get_player_rating_history(
        username: &str,
    ) -> Result<AllGamesRatingHistory, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://lichess.org/api/user/{}/rating-history",
                username
            ))
            .send()
            .await?;

        response.json::<AllGamesRatingHistory>().await
    }

    async fn get_player_profile(token: &str) -> Result<LichessUser, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://lichess.org/api/account")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        response.json::<LichessUser>().await
    }

    pub async fn get_last_games_analysis(
        token: &str,
        username: &str,
        number_of_games: usize,
    ) -> Result<Vec<Game>, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://lichess.org/api/games/user/{}?opening=true&perfType=rapid&accuracy=true&max={}",
                username,
                number_of_games
            ))
            .header("Accept", "application/x-ndjson")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await;

        let r = response.unwrap().text().await.unwrap();
        let result_stream = Deserializer::from_str(r.as_str()).into_iter::<Game>();

        Ok(result_stream.flatten().collect())
    }

    pub async fn get_player_data(
        token: &str,
        username: &str,
    ) -> Result<(LichessUser, AllGamesRatingHistory), reqwest::Error> {
        let profile_future = ChessModule::get_player_profile(token);
        let rating_history_future = ChessModule::get_player_rating_history(username);

        let (profile, rating_history) = try_join!(profile_future, rating_history_future)?;

        Ok((profile, rating_history))
    }
}
