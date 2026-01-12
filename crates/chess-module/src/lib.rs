use crate::types::{AllGamesRatingHistory, LichessUser};
mod types;

pub struct ChessModule;

impl ChessModule {
    pub fn get_player_stats_by_game_mode(token: &str) -> Option<LichessUser> {
        let response = std::thread::scope(|s| {
            s.spawn(|| {
                minreq::get("https://lichess.org/api/account")
                    .with_header("Authorization", format!("Bearer {}", token))
                    .send()
            })
            .join()
            .unwrap()
        });
        match response {
            Ok(stats) => Some(stats.json::<LichessUser>().unwrap()),
            Err(_) => None,
        }
    }

    fn get_player_rating_history(username: &str) -> Option<AllGamesRatingHistory> {
        let response = std::thread::scope(|s| {
            s.spawn(|| {
                minreq::get(format!(
                    "https://lichess.org/api/user/{}/rating-history",
                    username
                ))
                .send()
            })
            .join()
            .unwrap()
        });
        match response {
            Ok(history) => Some(history.json::<AllGamesRatingHistory>().unwrap()),
            Err(_) => None,
        }
    }

    fn get_player_profile(token: &str) -> Option<LichessUser> {
        let response = std::thread::scope(|s| {
            s.spawn(|| {
                minreq::get("https://lichess.org/api/account")
                    .with_header("Authorization", format!("Bearer {}", token))
                    .send()
            })
            .join()
            .unwrap()
        });
        match response {
            Ok(player) => Some(player.json::<LichessUser>().unwrap()),
            Err(_) => None,
        }
    }

    pub fn get_player_data(
        token: &str,
        username: &str,
    ) -> Option<(LichessUser, AllGamesRatingHistory)> {
        let profile = ChessModule::get_player_profile(token);
        let rating_history = ChessModule::get_player_rating_history(username);

        match (profile, rating_history) {
            (Some(p), Some(h)) => Some((p, h)),
            _ => None,
        }
    }
}
