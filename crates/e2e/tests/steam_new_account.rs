use e2e::TestApp;
use e2e::mocks::mount_steam;
use serde_json::json;

#[tokio::test]
async fn steam_page_handles_an_account_without_games() {
    let app = TestApp::spawn().await;
    mount_steam(&app, json!([]), 0).await;

    let page = app.htmx_get("/steam").await;

    assert_eq!(page.status, 200);
    assert!(page.body.contains("FixtureGamer"));
}
