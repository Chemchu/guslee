use tokio::try_join;

use crate::types::{AllGamesRatingHistory, LichessUser};
mod types;

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
