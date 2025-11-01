use serde::{Deserialize, Serialize};

pub struct ChessModule;

impl ChessModule {
    pub fn get_player_stats_by_game_mode(
        player_name: &str,
        game_mode: &str,
    ) -> Option<PlayerStats> {
        let response = std::thread::scope(|s| {
            s.spawn(|| {
                minreq::get(format!(
                    "https://www.chess.com/callback/stats/live/{}/{}",
                    game_mode, player_name
                ))
                .with_header(
                    "User-Agent",
                    "Mozilla/5.0 (Windows NT 11.0; Win64; x64) AppleWebKit/537.36",
                )
                .with_header("Accept", "application/json")
                .with_header("Accept-Language", "en-US,en;q=0.9")
                .send()
            })
            .join()
            .unwrap()
        });

        match response {
            Ok(stats) => Some(stats.json::<PlayerStats>().unwrap()),
            Err(_) => None,
        }
    }

    pub fn get_player_data(player_name: &str) -> Option<PlayerData> {
        let response = std::thread::scope(|s| {
            s.spawn(|| {
                minreq::get(format!("https://api.chess.com/pub/player/{}", player_name))
                    .with_header(
                        "User-Agent",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
                    )
                    .with_header("Accept", "application/json")
                    .with_header("Accept-Language", "en-US,en;q=0.9")
                    .send()
            })
            .join()
            .unwrap()
        });

        match response {
            Ok(player) => Some(player.json::<PlayerData>().unwrap()),
            Err(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub stats: Stats,
    pub progress: u32,
    pub rank: u64,
    pub percentile: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    #[serde(rename = "white_game_count")]
    pub white_game_count: u32,
    #[serde(rename = "black_game_count")]
    pub black_game_count: u32,
    #[serde(rename = "win_count")]
    pub win_count: u32,
    #[serde(rename = "draw_count")]
    pub draw_count: u32,
    #[serde(rename = "loss_count")]
    pub loss_count: u32,
    pub history: Vec<RatingHistory>,
    pub count: u32,
    #[serde(rename = "rated_count")]
    pub rated_count: u32,
    #[serde(rename = "opponent_rating_avg")]
    pub opponent_rating_avg: f64,
    #[serde(rename = "opponent_rating_win_avg")]
    pub opponent_rating_win_avg: f64,
    #[serde(rename = "opponent_rating_draw_avg")]
    pub opponent_rating_draw_avg: f64,
    #[serde(rename = "opponent_rating_loss_avg")]
    pub opponent_rating_loss_avg: f64,
    #[serde(rename = "white_win_count")]
    pub white_win_count: u32,
    #[serde(rename = "white_draw_count")]
    pub white_draw_count: u32,
    #[serde(rename = "white_loss_count")]
    pub white_loss_count: u32,
    #[serde(rename = "black_win_count")]
    pub black_win_count: u32,
    #[serde(rename = "black_draw_count")]
    pub black_draw_count: u32,
    #[serde(rename = "black_loss_count")]
    pub black_loss_count: u32,
    #[serde(rename = "rating_last")]
    pub rating_last: u32,
    #[serde(rename = "rating_first")]
    pub rating_first: u32,
    #[serde(rename = "rating_max")]
    pub rating_max: u32,
    #[serde(rename = "rating_max_timestamp")]
    pub rating_max_timestamp: u64,
    #[serde(rename = "moves_count")]
    pub moves_count: u32,
    #[serde(rename = "streak_last")]
    pub streak_last: i32,
    #[serde(rename = "streak_max")]
    pub streak_max: u32,
    #[serde(rename = "streak_max_timestamp")]
    pub streak_max_timestamp: u64,
    #[serde(rename = "opponent_rating_max")]
    pub opponent_rating_max: u32,
    #[serde(rename = "opponent_rating_max_timestamp")]
    pub opponent_rating_max_timestamp: u64,
    #[serde(rename = "opponent_rating_max_uuid")]
    pub opponent_rating_max_uuid: String,
    #[serde(rename = "accuracy_count")]
    pub accuracy_count: u32,
    #[serde(rename = "accuracy_avg")]
    pub accuracy_avg: f64,
    #[serde(rename = "starting_day")]
    pub starting_day: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatingHistory {
    pub timestamp: u64,
    pub rating: u32,
    pub day_close_rating: u32,
    pub day: u32,
}

#[derive(Debug, Deserialize)]
pub struct PlayerData {
    pub avatar: Option<String>,
    pub player_id: u64,
    #[serde(rename = "@id")]
    pub id: String,
    pub url: String,
    pub name: Option<String>, // Some players don't have a real name set
    pub username: String,
    pub title: Option<String>, // Not all players have titles (GM, IM, etc.)
    pub followers: u64,
    pub country: String,
    pub location: Option<String>,
    pub last_online: u64, // Unix timestamp
    pub joined: u64,      // Unix timestamp
    pub status: String,
    pub is_streamer: bool,
    pub twitch_url: Option<String>,
    pub verified: bool,
    pub league: Option<String>,
}
