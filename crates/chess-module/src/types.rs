use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRatingHistory {
    pub name: String,
    pub points: Vec<RatingPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RatingPoint(pub [i32; 4]);

impl RatingPoint {
    pub fn year(&self) -> i32 {
        self.0[0]
    }

    pub fn month(&self) -> i32 {
        self.0[1]
    }

    pub fn day(&self) -> i32 {
        self.0[2]
    }

    pub fn rating(&self) -> i32 {
        self.0[3]
    }

    pub fn to_timestamp_ms(&self) -> i64 {
        use chrono::NaiveDate;

        let month = (self.month() + 1) as u32;
        let year = self.year();
        let day = self.day() as u32;

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            date.and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc().timestamp_millis())
                .unwrap_or(0)
        } else {
            0
        }
    }
}

pub type AllGamesRatingHistory = Vec<GameRatingHistory>;

impl GameRatingHistory {
    pub fn current_rating(&self) -> Option<i32> {
        self.points.last().map(|p| p.rating())
    }

    pub fn peak_rating(&self) -> Option<i32> {
        self.points.iter().map(|p| p.rating()).max()
    }

    pub fn lowest_rating(&self) -> Option<i32> {
        self.points.iter().map(|p| p.rating()).min()
    }

    pub fn recent_points(&self, days: i64) -> Vec<&RatingPoint> {
        use chrono::Utc;

        let now = Utc::now().timestamp_millis();
        let cutoff = now - (days * 24 * 60 * 60 * 1000);

        self.points
            .iter()
            .filter(|p| p.to_timestamp_ms() >= cutoff)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_rating_history() {
        let json = r#"
        [
            {
                "name": "Bullet",
                "points": [
                    [2011, 0, 8, 1472],
                    [2011, 0, 9, 1332],
                    [2011, 8, 12, 1314]
                ]
            },
            {
                "name": "Blitz",
                "points": [
                    [2011, 7, 29, 1332]
                ]
            }
        ]
        "#;

        let history: AllGamesRatingHistory = serde_json::from_str(json).unwrap();

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].name, "Bullet");
        assert_eq!(history[0].points.len(), 3);
        assert_eq!(history[0].points[0].rating(), 1472);
        assert_eq!(history[0].current_rating(), Some(1314));
        assert_eq!(history[0].peak_rating(), Some(1472));
    }

    #[test]
    fn test_rating_point_accessors() {
        let point = RatingPoint([2011, 0, 8, 1472]);

        assert_eq!(point.year(), 2011);
        assert_eq!(point.month(), 0); // January (0-indexed)
        assert_eq!(point.day(), 8);
        assert_eq!(point.rating(), 1472);
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LichessUser {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perfs: Option<Perfs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<Title>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flair: Option<String>,
    #[serde(rename = "createdAt", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(rename = "tosViolation", skip_serializing_if = "Option::is_none")]
    pub tos_violation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
    #[serde(rename = "seenAt", skip_serializing_if = "Option::is_none")]
    pub seen_at: Option<i64>,
    #[serde(rename = "playTime", skip_serializing_if = "Option::is_none")]
    pub play_time: Option<PlayTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patron: Option<bool>,
    #[serde(rename = "patronColor", skip_serializing_if = "Option::is_none")]
    pub patron_color: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    // Extended fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<Count>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streamer: Option<Streamer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub followable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub following: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perfs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chess960: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atomic: Option<PerfStat>,
    #[serde(rename = "racingKings", skip_serializing_if = "Option::is_none")]
    pub racing_kings: Option<PerfStat>,
    #[serde(rename = "ultraBullet", skip_serializing_if = "Option::is_none")]
    pub ultra_bullet: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blitz: Option<PerfStat>,
    #[serde(rename = "kingOfTheHill", skip_serializing_if = "Option::is_none")]
    pub king_of_the_hill: Option<PerfStat>,
    #[serde(rename = "threeCheck", skip_serializing_if = "Option::is_none")]
    pub three_check: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antichess: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crazyhouse: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bullet: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correspondence: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub horde: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub puzzle: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classical: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rapid: Option<PerfStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storm: Option<MinigameStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub racer: Option<MinigameStat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streak: Option<MinigameStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfStat {
    pub games: u32,
    pub rating: u32,
    pub rd: u32,
    pub prog: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prov: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinigameStat {
    pub runs: u32,
    pub score: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Title {
    GM,
    Wgm,
    IM,
    Wim,
    FM,
    Wfm,
    NM,
    CM,
    Wcm,
    Wnm,
    LM,
    Bot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(rename = "realName", skip_serializing_if = "Option::is_none")]
    pub real_name: Option<String>,
    #[serde(rename = "fideRating", skip_serializing_if = "Option::is_none")]
    pub fide_rating: Option<u32>,
    #[serde(rename = "uscfRating", skip_serializing_if = "Option::is_none")]
    pub uscf_rating: Option<u32>,
    #[serde(rename = "ecfRating", skip_serializing_if = "Option::is_none")]
    pub ecf_rating: Option<u32>,
    #[serde(rename = "cfcRating", skip_serializing_if = "Option::is_none")]
    pub cfc_rating: Option<u32>,
    #[serde(rename = "rcfRating", skip_serializing_if = "Option::is_none")]
    pub rcf_rating: Option<u32>,
    #[serde(rename = "dsbRating", skip_serializing_if = "Option::is_none")]
    pub dsb_rating: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayTime {
    pub total: u32,
    pub tv: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Count {
    pub all: u32,
    pub rated: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai: Option<u32>,
    pub draw: u32,
    #[serde(rename = "drawH", skip_serializing_if = "Option::is_none")]
    pub draw_h: Option<u32>,
    pub loss: u32,
    #[serde(rename = "lossH", skip_serializing_if = "Option::is_none")]
    pub loss_h: Option<u32>,
    pub win: u32,
    #[serde(rename = "winH", skip_serializing_if = "Option::is_none")]
    pub win_h: Option<u32>,
    pub bookmark: u32,
    pub playing: u32,
    #[serde(rename = "import")]
    pub import_count: u32,
    pub me: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Streamer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitch: Option<StreamChannel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub youtube: Option<StreamChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChannel {
    pub channel: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub rated: bool,
    pub variant: Variant,
    pub speed: Speed,
    pub perf: String,
    pub created_at: i64,
    pub last_move_at: i64,
    pub status: Status,
    pub players: Players,
    pub source: Option<String>,
    pub initial_fen: Option<String>,
    pub winner: Option<Winner>,
    pub opening: Option<Opening>,
    pub moves: Option<String>,
    pub pgn: Option<String>,
    pub days_per_turn: Option<i32>,
    pub analysis: Option<Vec<MoveAnalysis>>,
    pub tournament: Option<String>,
    pub swiss: Option<String>,
    pub clock: Option<Clock>,
    pub clocks: Option<Vec<i64>>,
    pub division: Option<Division>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Players {
    pub white: Player,
    pub black: Player,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub user: Option<UserInfo>,
    pub rating: Option<i32>,
    pub rating_diff: Option<i32>,
    pub name: Option<String>,
    pub provisional: Option<bool>,
    pub ai_level: Option<i32>,
    pub analysis: Option<PlayerAnalysis>,
    pub team: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub flair: Option<String>,
    pub title: Option<Title>,
    pub patron_color: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerAnalysis {
    pub inaccuracy: i32,
    pub mistake: i32,
    pub blunder: i32,
    pub acpl: i32,
    pub accuracy: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Opening {
    pub eco: String,
    pub name: String,
    pub ply: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MoveAnalysis {
    pub eval: Option<i32>,
    pub mate: Option<i32>,
    pub best: Option<String>,
    pub variation: Option<String>,
    pub judgment: Option<Judgment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Judgment {
    pub name: JudgmentName,
    pub comment: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Clock {
    pub initial: i32,
    pub increment: i32,
    #[serde(rename = "totalTime")]
    pub total_time: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Division {
    pub middle: Option<i32>,
    pub end: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Variant {
    Standard,
    Chess960,
    Crazyhouse,
    Antichess,
    Atomic,
    Horde,
    KingOfTheHill,
    RacingKings,
    ThreeCheck,
    FromPosition,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Created,
    Started,
    Aborted,
    Mate,
    Resign,
    Stalemate,
    Timeout,
    Draw,
    Outoftime,
    Cheat,
    NoStart,
    UnknownFinish,
    InsufficientMaterialClaim,
    VariantEnd,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Winner {
    White,
    Black,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum JudgmentName {
    Inaccuracy,
    Mistake,
    Blunder,
}
