use serde::Deserialize;

pub struct SteamState {
    steam_token: String,
    steam_id: String,
    api_base_url: String,
}

#[derive(Deserialize)]
struct SteamProfileResponse {
    response: SteamProfileData,
}

#[derive(Deserialize)]
struct SteamProfileData {
    players: Vec<SteamPlayerData>,
}

#[derive(Deserialize)]
struct SteamPlayerData {
    personaname: String,
    realname: Option<String>,
    avatarfull: String,
    profileurl: String,
    steamid: String,
    personastate: u32,
    timecreated: Option<u64>,
    loccountrycode: Option<String>,
}

#[derive(Deserialize)]
struct PlayerLevelResponse {
    response: PlayerLevel,
}

#[derive(Deserialize)]
struct PlayerLevel {
    player_level: u32,
}

#[derive(Deserialize)]
struct OwnedGamesResponse {
    response: OwnedGamesData,
}

#[derive(Deserialize)]
struct OwnedGamesData {
    game_count: u32,
    games: Option<Vec<OwnedGameData>>,
}

#[derive(Deserialize)]
struct OwnedGameData {
    appid: u32,
    name: Option<String>,
    playtime_forever: u32,
    img_icon_url: Option<String>,
    playtime_2weeks: Option<u32>,
}

#[derive(Deserialize)]
struct RecentGamesResponse {
    response: RecentGamesData,
}

#[derive(Deserialize)]
struct RecentGamesData {
    games: Vec<RecentGameData>,
}

#[derive(Deserialize)]
struct RecentGameData {
    name: String,
    appid: u32,
    playtime_2weeks: u32,
    playtime_forever: u32,
    img_icon_url: String,
}

#[derive(Deserialize)]
struct PlayerAchievementsResponse {
    playerstats: PlayerAchievementsData,
}

#[derive(Deserialize)]
struct PlayerAchievementsData {
    achievements: Option<Vec<AchievementData>>,
}

#[derive(Deserialize)]
struct AchievementData {
    apiname: String,
    achieved: u32,
    unlocktime: u64,
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug)]
pub struct SteamProfile {
    pub personaname: String,
    pub realname: Option<String>,
    pub avatar_full: String,
    pub profileurl: String,
    pub steamid: String,
    pub personastate: u32,
    pub timecreated: Option<u64>,
    pub loccountrycode: Option<String>,
    pub level: u32,
    pub game_count: u32,
}

#[derive(Debug, Clone)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: bool,
    pub unlocktime: u64,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct RecentGame {
    pub name: String,
    pub appid: String,
    pub playtime_2weeks: u32,  // in minutes
    pub playtime_forever: u32, // in minutes
    pub img_icon_url: String,
    pub img_logo_url: String,
    pub achievements: Vec<Achievement>,
    pub total_achievements: u32,
    pub unlocked_achievements: u32,
    pub achievement_progress_percentage: f32,
}

#[derive(Debug)]
pub struct TopGame {
    pub name: String,
    pub appid: String,
    pub playtime_2weeks: Option<u32>,
    pub playtime_forever: u32, // in minutes
    pub img_icon_url: String,
    pub img_logo_url: String,
    pub achievements: Vec<Achievement>,
    pub total_achievements: u32,
    pub unlocked_achievements: u32,
    pub achievement_progress_percentage: f32,
}

pub const STEAM_API_BASE_URL: &str = "https://api.steampowered.com";

// App ids that are never shown in the top played games list
const EXCLUDED_APP_IDS: [u32; 2] = [431960, 323370];

impl SteamState {
    pub fn new(steam_token: String, steam_id: String) -> Self {
        Self {
            steam_token,
            steam_id,
            api_base_url: STEAM_API_BASE_URL.to_string(),
        }
    }

    // Overridable so tests can point the module at a mock server
    pub fn with_api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = api_base_url.into();
        self
    }

    pub async fn get_profile(&self) -> Result<SteamProfile, reqwest::Error> {
        let client = reqwest::Client::new();

        let profile_url = format!(
            "{}/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
            self.api_base_url, self.steam_token, self.steam_id
        );
        let profile_response: SteamProfileResponse = client
            .get(&profile_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let player = &profile_response.response.players[0];

        let level_url = format!(
            "{}/IPlayerService/GetSteamLevel/v1/?key={}&steamid={}",
            self.api_base_url, self.steam_token, self.steam_id
        );
        let level_response: PlayerLevelResponse = client
            .get(&level_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let games_url = format!(
            "{}/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}",
            self.api_base_url, self.steam_token, self.steam_id
        );
        let games_response: OwnedGamesResponse = client
            .get(&games_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(SteamProfile {
            personaname: player.personaname.clone(),
            realname: player.realname.clone(),
            avatar_full: player.avatarfull.clone(),
            profileurl: player.profileurl.clone(),
            steamid: player.steamid.clone(),
            personastate: player.personastate,
            timecreated: player.timecreated,
            loccountrycode: player.loccountrycode.clone(),
            level: level_response.response.player_level,
            game_count: games_response.response.game_count,
        })
    }

    pub async fn get_recent_games(
        &self,
        recent_games_count: u8,
    ) -> Result<Vec<RecentGame>, reqwest::Error> {
        let client = reqwest::Client::new();

        let recent_url = format!(
            "{}/IPlayerService/GetRecentlyPlayedGames/v1/?key={}&steamid={}&count={}",
            self.api_base_url, self.steam_token, self.steam_id, recent_games_count
        );
        let recent_response: RecentGamesResponse = client
            .get(&recent_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let achievements_results = futures::future::join_all(
            recent_response
                .response
                .games
                .iter()
                .map(|game| self.get_achievements(game.appid)),
        )
        .await;

        let games: Vec<RecentGame> = recent_response
            .response
            .games
            .iter()
            .zip(achievements_results)
            .map(|(game, achievements)| {
                let progress = AchievementProgress::from_achievements(&achievements);

                RecentGame {
                    name: game.name.clone(),
                    appid: game.appid.to_string(),
                    playtime_2weeks: game.playtime_2weeks,
                    playtime_forever: game.playtime_forever,
                    img_icon_url: game.img_icon_url.clone(),
                    img_logo_url: header_image_url(game.appid),
                    achievements,
                    total_achievements: progress.total,
                    unlocked_achievements: progress.unlocked,
                    achievement_progress_percentage: progress.percentage,
                }
            })
            .collect();

        Ok(games)
    }

    pub async fn get_top_played_games(
        &self,
        top_games_count: u8,
    ) -> Result<Vec<TopGame>, reqwest::Error> {
        let client = reqwest::Client::new();

        let games_url = format!(
            "{}/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true",
            self.api_base_url, self.steam_token, self.steam_id
        );
        let games_response: OwnedGamesResponse = client
            .get(&games_url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let owned_games = select_top_games(
            games_response.response.games.unwrap_or_default(),
            top_games_count as usize,
        );

        let achievements_results = futures::future::join_all(
            owned_games
                .iter()
                .map(|game| self.get_achievements(game.appid)),
        )
        .await;

        let games: Vec<TopGame> = owned_games
            .iter()
            .zip(achievements_results)
            .map(|(game, achievements)| {
                let progress = AchievementProgress::from_achievements(&achievements);

                TopGame {
                    name: game
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("Game {}", game.appid)),
                    appid: game.appid.to_string(),
                    playtime_forever: game.playtime_forever,
                    img_icon_url: game.img_icon_url.clone().unwrap_or_default(),
                    img_logo_url: header_image_url(game.appid),
                    playtime_2weeks: game.playtime_2weeks,
                    achievements,
                    total_achievements: progress.total,
                    unlocked_achievements: progress.unlocked,
                    achievement_progress_percentage: progress.percentage,
                }
            })
            .collect();

        Ok(games)
    }

    // Games without achievements (or private stats) return an error from Steam, so any
    // failure is treated as "no achievements" instead of failing the whole page
    async fn get_achievements(&self, appid: u32) -> Vec<Achievement> {
        let url = format!(
            "{}/ISteamUserStats/GetPlayerAchievements/v1/?key={}&steamid={}&appid={}",
            self.api_base_url, self.steam_token, self.steam_id, appid
        );

        let response = match reqwest::Client::new().get(&url).send().await {
            Ok(response) => response,
            Err(_) => return Vec::new(),
        };

        match response.json::<PlayerAchievementsResponse>().await {
            Ok(ach_response) => ach_response
                .playerstats
                .achievements
                .unwrap_or_default()
                .into_iter()
                .map(|ach| Achievement {
                    apiname: ach.apiname,
                    achieved: ach.achieved == 1,
                    unlocktime: ach.unlocktime,
                    name: ach.name,
                    description: ach.description,
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct AchievementProgress {
    total: u32,
    unlocked: u32,
    percentage: f32,
}

impl AchievementProgress {
    fn from_achievements(achievements: &[Achievement]) -> Self {
        let total = achievements.len() as u32;
        let unlocked = achievements.iter().filter(|a| a.achieved).count() as u32;
        let percentage = if total > 0 {
            (unlocked as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        Self {
            total,
            unlocked,
            percentage,
        }
    }
}

// Most played first, skipping non-games and games never launched
fn select_top_games(games: Vec<OwnedGameData>, count: usize) -> Vec<OwnedGameData> {
    let mut games: Vec<OwnedGameData> = games
        .into_iter()
        .filter(|game| !EXCLUDED_APP_IDS.contains(&game.appid))
        .filter(|game| game.playtime_forever > 0)
        .collect();

    games.sort_by_key(|game| std::cmp::Reverse(game.playtime_forever));
    games.truncate(count);
    games
}

fn header_image_url(appid: u32) -> String {
    format!(
        "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/header.jpg",
        appid
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned_game(appid: u32, playtime_forever: u32) -> OwnedGameData {
        OwnedGameData {
            appid,
            name: Some(format!("Game {}", appid)),
            playtime_forever,
            img_icon_url: None,
            playtime_2weeks: None,
        }
    }

    fn achievement(achieved: bool) -> Achievement {
        Achievement {
            apiname: "ACH".to_string(),
            achieved,
            unlocktime: 0,
            name: None,
            description: None,
        }
    }

    #[test]
    fn top_games_are_sorted_by_playtime_and_truncated() {
        let games = vec![owned_game(1, 10), owned_game(2, 300), owned_game(3, 50)];

        let top: Vec<u32> = select_top_games(games, 2).iter().map(|g| g.appid).collect();

        assert_eq!(top, vec![2, 3]);
    }

    #[test]
    fn top_games_skip_excluded_and_never_played_games() {
        let games = vec![
            owned_game(EXCLUDED_APP_IDS[0], 10_000),
            owned_game(EXCLUDED_APP_IDS[1], 9_000),
            owned_game(7, 0),
            owned_game(8, 5),
        ];

        let top: Vec<u32> = select_top_games(games, 5).iter().map(|g| g.appid).collect();

        assert_eq!(top, vec![8]);
    }

    #[test]
    fn achievement_progress_counts_unlocked_achievements() {
        let achievements = vec![
            achievement(true),
            achievement(false),
            achievement(true),
            achievement(false),
        ];

        let progress = AchievementProgress::from_achievements(&achievements);

        assert_eq!(
            progress,
            AchievementProgress {
                total: 4,
                unlocked: 2,
                percentage: 50.0
            }
        );
    }

    #[test]
    fn achievement_progress_is_zero_without_achievements() {
        let progress = AchievementProgress::from_achievements(&[]);

        assert_eq!(progress.total, 0);
        assert_eq!(progress.percentage, 0.0);
    }

    #[test]
    fn header_image_points_to_the_steam_cdn() {
        assert_eq!(
            header_image_url(620),
            "https://cdn.cloudflare.steamstatic.com/steam/apps/620/header.jpg"
        );
    }

    #[test]
    fn steam_state_defaults_to_the_public_api() {
        let state = SteamState::new("key".into(), "id".into());
        assert_eq!(state.api_base_url, STEAM_API_BASE_URL);

        let state = state.with_api_base_url("http://localhost:1234");
        assert_eq!(state.api_base_url, "http://localhost:1234");
    }
}
