use serde::Deserialize;

pub struct SteamState {
    steam_token: String,
    steam_id: String,
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

impl SteamState {
    pub fn new(steam_token: String, steam_id: String) -> Self {
        Self {
            steam_token,
            steam_id,
        }
    }

    pub async fn get_profile(&self) -> Result<SteamProfile, reqwest::Error> {
        let client = reqwest::Client::new();

        let profile_url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
            self.steam_token, self.steam_id
        );
        let profile_response: SteamProfileResponse =
            client.get(&profile_url).send().await?.json().await?;

        let player = &profile_response.response.players[0];

        let level_url = format!(
            "https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key={}&steamid={}",
            self.steam_token, self.steam_id
        );
        let level_response: PlayerLevelResponse =
            client.get(&level_url).send().await?.json().await?;

        let games_url = format!(
            "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}",
            self.steam_token, self.steam_id
        );
        let games_response: OwnedGamesResponse =
            client.get(&games_url).send().await?.json().await?;

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

        let url = format!(
            "https://api.steampowered.com/IPlayerService/GetRecentlyPlayedGames/v1/?key={}&steamid={}&count={}",
            self.steam_token, self.steam_id, recent_games_count
        );

        let response: RecentGamesResponse = client.get(&url).send().await?.json().await?;
        let games: Vec<RecentGame> = response
            .response
            .games
            .iter()
            .map(|game| RecentGame {
                name: game.name.clone(),
                appid: game.appid.to_string(),
                playtime_2weeks: game.playtime_2weeks,
                playtime_forever: game.playtime_forever,
                img_icon_url: game.img_icon_url.clone(),
            })
            .collect();

        Ok(games)
    }
}

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

pub struct RecentGame {
    pub name: String,
    pub appid: String,
    pub playtime_2weeks: u32,  // in minutes
    pub playtime_forever: u32, // in minutes
    pub img_icon_url: String,
}
