use e2e::{SPOTIFY_REFRESH_TOKEN, TestApp};
use serde_json::json;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const ACCESS_TOKEN: &str = "fresh-access-token";

fn profile(images: serde_json::Value) -> serde_json::Value {
    json!({
        "display_name": "Fixture Listener",
        "id": "listener",
        "images": images,
        "followers": {"total": 321},
        "external_urls": {"spotify": "https://open.spotify.com/user/listener"},
        "product": "premium"
    })
}

fn tracks(id: &str) -> serde_json::Value {
    json!({
        "items": [{
            "name": "Track",
            "id": id,
            "artists": [{"name": "Track Artist", "id": "a1", "external_urls": {"spotify": "https://open.spotify.com/artist/a1"}}],
            "album": {"name": "Album", "images": [{"url": "https://i.scdn.co/album.jpg", "height": 64, "width": 64}], "external_urls": {"spotify": "https://open.spotify.com/album/b1"}},
            "external_urls": {"spotify": format!("https://open.spotify.com/track/{id}")},
            "preview_url": null,
            "duration_ms": 200000
        }],
        "total": 1, "limit": 5, "offset": 0
    })
}

fn artists(name: &str) -> serde_json::Value {
    json!({
        "items": [{
            "external_urls": {"spotify": "https://open.spotify.com/artist/x"},
            "followers": {"href": null, "total": 1000},
            "genres": ["shoegaze"],
            "href": "https://api.spotify.com/v1/artists/x",
            "id": "x",
            "images": [{"url": "https://i.scdn.co/artist.jpg", "height": 160, "width": 160}],
            "name": name,
            "popularity": 50,
            "type": "artist",
            "uri": "spotify:artist:x"
        }],
        "total": 1, "limit": 5, "offset": 0,
        "href": "https://api.spotify.com/v1/me/top/artists", "previous": null, "next": null
    })
}

async fn mount_spotify(app: &TestApp) {
    let ok = |body: serde_json::Value| ResponseTemplate::new(200).set_body_json(body);
    let authorized = || header("Authorization", format!("Bearer {ACCESS_TOKEN}"));

    Mock::given(method("POST"))
        .and(path("/api/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .and(body_string_contains(format!(
            "refresh_token={SPOTIFY_REFRESH_TOKEN}"
        )))
        .respond_with(ok(json!({
            "access_token": ACCESS_TOKEN, "token_type": "Bearer",
            "expires_in": 3600, "scope": "user-top-read"
        })))
        .mount(&app.spotify)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/me"))
        .and(authorized())
        .respond_with(ok(profile(
            json!([{"url": "https://i.scdn.co/me.jpg", "height": 300, "width": 300}]),
        )))
        .mount(&app.spotify)
        .await;
    for (range, suffix) in [("long_term", "all time"), ("medium_term", "this half")] {
        Mock::given(method("GET"))
            .and(path("/v1/me/top/tracks"))
            .and(query_param("time_range", range))
            .and(query_param("limit", "5"))
            .and(authorized())
            .respond_with(ok(tracks(range)))
            .mount(&app.spotify)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/me/top/artists"))
            .and(query_param("time_range", range))
            .and(authorized())
            .respond_with(ok(artists(&format!("Artist of {suffix}"))))
            .mount(&app.spotify)
            .await;
    }
}

#[tokio::test]
async fn music_pages_render_spotify_data_and_retry_after_failures() {
    let app = TestApp::spawn().await;

    Mock::given(method("POST"))
        .and(path("/api/token"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&app.spotify)
        .await;
    assert!(
        app.htmx_get("/music/profile")
            .await
            .body
            .contains("Error loading spotify")
    );
    assert!(
        app.htmx_get("/music/user-top/tracks/long_term")
            .await
            .body
            .contains("Error loading Spotify item")
    );

    app.spotify.reset().await;
    mount_spotify(&app).await;

    let profile_page = app.htmx_get("/music/profile").await;
    assert!(!profile_page.is_full_page());
    assert!(profile_page.body.contains("Fixture Listener"));
    assert!(profile_page.body.contains("Premium Subscriber"));
    assert!(profile_page.body.contains(">321<"));
    assert!(app.get("/music/profile").await.is_full_page());

    let top_tracks = app.htmx_get("/music/user-top/tracks/long_term").await.body;
    assert!(top_tracks.contains("https://open.spotify.com/embed/track/long_term?"));
    let top_artists = app.htmx_get("/music/user-top/artists/long_term").await.body;
    assert!(top_artists.contains("Artist of all time"));
    assert!(
        app.htmx_get("/music/user-top/podcasts/long_term")
            .await
            .body
            .contains("Error loading Spotify item")
    );

    let cycled = app
        .htmx_get("/music/time-range/artists/long_term/next")
        .await
        .body;
    assert!(cycled.contains("Last 6 months"));
    assert!(cycled.contains("hx-get=\"/music/time-range/artists/medium_term/next\""));
    assert!(cycled.contains("id=\"top-artists\" hx-swap-oob=\"innerHTML\""));
    assert!(cycled.contains("Artist of this half"));

    Mock::given(method("GET"))
        .and(path("/v1/me/top/artists"))
        .and(query_param("time_range", "short_term"))
        .respond_with(ResponseTemplate::new(200).set_body_json({
            let mut body = artists("Faceless artist");
            body["items"][0]["images"] = json!([]);
            body
        }))
        .mount(&app.spotify)
        .await;
    let faceless = app.htmx_get("/music/user-top/artists/short_term").await;
    assert_eq!(faceless.status, 200);
    assert!(faceless.body.contains("Faceless artist"));

    app.spotify.reset().await;
    mount_spotify_without_avatar(&app).await;
    let no_avatar = app.htmx_get("/music/profile").await;
    assert_eq!(no_avatar.status, 200);
    assert!(no_avatar.body.contains("Fixture Listener"));
}

async fn mount_spotify_without_avatar(app: &TestApp) {
    Mock::given(method("POST"))
        .and(path("/api/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": ACCESS_TOKEN, "token_type": "Bearer",
            "expires_in": 3600, "scope": "user-top-read"
        })))
        .mount(&app.spotify)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(profile(json!([]))))
        .mount(&app.spotify)
        .await;
}
