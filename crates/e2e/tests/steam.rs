use e2e::mocks::mount_steam;
use e2e::{STEAM_ID, TestApp};
use serde_json::json;
use wiremock::matchers::method;
use wiremock::{Mock, ResponseTemplate};

// One test function on purpose: the Steam page is cached process-wide (see the harness docs)
#[tokio::test]
async fn steam_page_renders_profile_and_games_and_retries_after_failures() {
    let app = TestApp::spawn().await;

    // 1. Steam is down: an error message is shown and not cached
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&app.steam)
        .await;
    let down = app.htmx_get("/steam").await;
    assert_eq!(down.status, 200);
    assert!(down.body.contains("Error loading Steam page"));

    // 2. Steam recovers
    app.steam.reset().await;
    mount_steam(
        &app,
        json!([
            {"appid": 431960, "name": "Excluded App", "playtime_forever": 99_999},
            {"appid": 10, "name": "Most Played", "playtime_forever": 6_000},
            {"appid": 20, "name": "Recent Adventure", "playtime_forever": 90},
            {"appid": 30, "name": "Never Launched", "playtime_forever": 0}
        ]),
        4,
    )
    .await;
    let page = app.htmx_get("/steam").await;
    let body = &page.body;

    assert!(!page.is_full_page());
    assert!(body.contains("FixtureGamer"));
    assert!(body.contains("Fixture Real Name"));
    assert!(body.contains("Level 42"));
    assert!(body.contains(&format!("Steam ID: {STEAM_ID}")));

    // Top games: most played first, excluded and never launched games left out
    assert!(!body.contains("Excluded App"));
    assert!(!body.contains("Never Launched"));
    assert!(body.find("Most Played").unwrap() < body.rfind("Recent Adventure").unwrap());
    assert!(body.contains("100h"), "6000 minutes");
    assert!(body.contains("1.5h"), "90 minutes");
    assert!(body.contains("https://cdn.cloudflare.steamstatic.com/steam/apps/10/header.jpg"));

    // Achievements: 1 of 2 for appid 20 (in both lists); appid 10's request fails, which
    // must not break the page and simply shows no progress bar
    assert_eq!(body.matches("width: 50%").count(), 2);
    assert_eq!(body.matches("style=\"width: ").count(), 2);
    assert!(body.contains("1/2</span>"));

    // 3. Direct visits get the layout around the same content
    assert!(app.get("/steam").await.is_full_page());
}
