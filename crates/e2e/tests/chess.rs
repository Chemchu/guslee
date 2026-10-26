use e2e::{LICHESS_TOKEN, LICHESS_USERNAME, TestApp};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

fn account(rapid_rating: u32) -> serde_json::Value {
    json!({
        "id": LICHESS_USERNAME.to_lowercase(),
        "username": LICHESS_USERNAME,
        "perfs": {
            "blitz": {"games": 10, "rating": 1111, "rd": 60, "prog": 0},
            "rapid": {"games": 250, "rating": rapid_rating, "rd": 45, "prog": 12},
            "kingOfTheHill": {"games": 3, "rating": 1333, "rd": 120, "prog": 0}
        },
        "count": {
            "all": 200, "rated": 190, "draw": 20, "loss": 80, "win": 100,
            "bookmark": 0, "playing": 0, "import": 0, "me": 0
        }
    })
}

fn rating_history() -> serde_json::Value {
    json!([
        {"name": "Rapid", "points": [[2024, 0, 1, 1500], [2024, 5, 1, 1777], [2024, 11, 1, 1640]]},
        {"name": "King of the Hill", "points": [[2024, 2, 1, 1333]]}
    ])
}

fn game(id: &str, white: &str, black: &str, opening: &str) -> String {
    json!({
        "id": id, "rated": true, "variant": "standard", "speed": "rapid", "perf": "rapid",
        "createdAt": 1_700_000_000_000_i64, "lastMoveAt": 1_700_000_600_000_i64,
        "status": "mate", "winner": "white",
        "players": {
            "white": {"user": {"id": white.to_lowercase(), "name": white}, "rating": 1600},
            "black": {"user": {"id": black.to_lowercase(), "name": black}, "rating": 1590}
        },
        "opening": {"eco": "C50", "name": opening, "ply": 7}
    })
    .to_string()
}

async fn mount_lichess(app: &TestApp, rapid_rating: u32) {
    Mock::given(method("GET"))
        .and(path("/api/account"))
        .and(header("Authorization", format!("Bearer {LICHESS_TOKEN}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(account(rapid_rating)))
        .mount(&app.lichess)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/api/user/{LICHESS_USERNAME}/rating-history")))
        .respond_with(ResponseTemplate::new(200).set_body_json(rating_history()))
        .mount(&app.lichess)
        .await;
}

#[tokio::test]
async fn chess_pages_render_lichess_data_and_retry_after_failures() {
    let app = TestApp::spawn().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&app.lichess)
        .await;
    let down = app.htmx_get("/chess/stats/rapid").await;
    assert_eq!(down.status, 200);
    assert!(
        down.body.contains("Page Not Found"),
        "fallback page expected"
    );

    app.lichess.reset().await;
    mount_lichess(&app, 1640).await;
    let rapid = app.htmx_get("/chess/stats/rapid").await;
    assert!(!rapid.is_full_page());
    assert!(rapid.body.contains(">1640<"), "current rapid rating");
    assert!(
        rapid.body.contains(">1777<"),
        "peak from the rating history"
    );
    assert!(rapid.body.contains(">250<"), "rapid games played");
    assert!(rapid.body.contains("50.00%"), "100 wins out of 200 games");
    assert!(rapid.body.contains("2024-06-01"));

    app.lichess.reset().await;
    mount_lichess(&app, 1999).await;
    let cached = app.htmx_get("/chess/stats/rapid").await;
    assert!(cached.body.contains(">1640<"));
    assert!(!cached.body.contains(">1999<"));

    assert!(app.get("/chess/stats/rapid").await.is_full_page());

    let koth = app.htmx_get("/chess/stats/kingOfTheHill").await;
    assert!(koth.body.contains(">1333<"));
    assert!(!koth.body.contains(">1111<"));

    Mock::given(method("GET"))
        .and(path(format!("/api/games/user/{LICHESS_USERNAME}")))
        .and(query_param("max", "10"))
        .and(header("Accept", "application/x-ndjson"))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!(
            "{}\n{}\n{}\n",
            game(
                "g1",
                LICHESS_USERNAME,
                "Rival",
                "Italian Game: Giuoco Piano"
            ),
            game("g2", LICHESS_USERNAME, "Rival", "Italian Game: Two Knights"),
            game("g3", "Rival", LICHESS_USERNAME, "Sicilian Defense: Najdorf"),
        )))
        .mount(&app.lichess)
        .await;
    let analysis = app.htmx_get("/chess/games/analysis").await;
    assert_eq!(analysis.status, 200);
    assert!(analysis.body.contains("Game Analysis"));
    assert!(analysis.body.contains("#g1 played as white"));
    assert!(analysis.body.contains("#g3 played as black"));
    let white_card = &analysis.body[analysis.body.find("Italian Game<").unwrap()..];
    assert!(white_card.starts_with("Italian Game<"));
    assert!(white_card.contains("<span class=\"text-primary-color\">2</span> times recently"));
}
