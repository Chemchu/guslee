use serde::Deserialize;

pub struct ChessModule;

impl ChessModule {
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
