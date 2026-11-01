use e2e::TestApp;

async fn get_with_language(app: &TestApp, path: &str, accept_language: &str) -> e2e::Response {
    app.send(
        app.client
            .get(app.url(path))
            .header("Accept-Language", accept_language),
    )
    .await
}

#[tokio::test]
async fn posts_follow_the_accept_language_header() {
    let app = TestApp::spawn().await;

    let spanish = get_with_language(&app, "/posts/notes/fog", "es-ES,es;q=0.9,en;q=0.8").await;
    let english = get_with_language(&app, "/posts/notes/fog", "en-GB,en;q=0.9").await;

    assert_eq!(spanish.status, 200);
    assert!(
        spanish
            .body
            .contains(r#"<h1 id="la-niebla">La niebla</h1>"#)
    );
    assert!(spanish.body.contains("<title>Notas sobre la niebla"));
    assert!(spanish.body.contains(r#"lang="es""#));
    assert_eq!(spanish.header("content-language"), Some("es"));

    assert!(english.body.contains(r#"<h1 id="the-fog">The fog</h1>"#));
    assert_eq!(english.header("content-language"), Some("en"));
}

#[tokio::test]
async fn untranslated_posts_fall_back_to_english() {
    let app = TestApp::spawn().await;

    let response = get_with_language(&app, "/posts/notes/letters", "es").await;

    assert_eq!(response.status, 200);
    assert_eq!(response.header("content-language"), Some("en"));
}

#[tokio::test]
async fn post_pages_vary_on_the_language_hints() {
    let app = TestApp::spawn().await;

    let response = app.get("/posts/notes/fog").await;

    let vary = response
        .headers
        .get_all("vary")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    assert!(vary.contains("Accept-Language, Cookie"), "vary was {vary}");
}

#[tokio::test]
async fn the_lang_query_overrides_the_header_and_is_remembered() {
    let app = TestApp::spawn().await;

    let chosen = get_with_language(&app, "/posts/notes/fog?lang=en", "es").await;
    assert_eq!(chosen.header("content-language"), Some("en"));
    assert!(chosen.header("set-cookie").unwrap().starts_with("lang=en;"));

    let remembered = app
        .send(
            app.client
                .get(app.url("/posts/notes/fog"))
                .header("Accept-Language", "es")
                .header("Cookie", "lang=en"),
        )
        .await;
    assert_eq!(remembered.header("content-language"), Some("en"));
}

#[tokio::test]
async fn htmx_fragments_and_side_panels_are_localized_too() {
    let app = TestApp::spawn().await;
    let htmx_get = |path: &'static str| {
        let app = &app;
        async move {
            app.send(
                app.client
                    .get(app.url(path))
                    .header("HX-Request", "true")
                    .header("Accept-Language", "es"),
            )
            .await
        }
    };

    assert!(
        htmx_get("/posts/notes/fog")
            .await
            .body
            .contains("La niebla")
    );
    assert!(
        htmx_get("/metadata/notes/fog")
            .await
            .body
            .contains("Todo sobre la niebla")
    );
    assert!(
        htmx_get("/preview/notes/fog")
            .await
            .body
            .contains("La niebla")
    );
}

#[tokio::test]
async fn search_results_are_cached_per_language() {
    let app = TestApp::spawn().await;
    let search = |lang: &'static str| {
        let app = &app;
        async move {
            app.send(
                app.client
                    .get(app.url("/search?query=niebla"))
                    .header("HX-Request", "true")
                    .header("Accept-Language", lang),
            )
            .await
            .body
        }
    };

    assert!(search("es").await.contains("Notas sobre la niebla"));
    let english = search("en").await;
    assert!(english.contains("/posts/notes/fog"));
    assert!(english.contains("Notes about the fog"));
}

#[tokio::test]
async fn the_switcher_lists_translated_titles_once() {
    let app = TestApp::spawn().await;

    let body = get_with_language(&app, "/switcher.json", "es").await.body;

    assert!(body.contains("Notas sobre la niebla"));
    assert!(!body.contains("Notes about the fog"));
    assert_eq!(body.matches(r#""path":"notes/fog""#).count(), 1);
}

#[tokio::test]
async fn translated_posts_advertise_their_language_versions() {
    let app = TestApp::spawn().await;
    let fog = format!("{}/posts/notes/fog", app.address);

    let spanish = get_with_language(&app, "/posts/notes/fog", "es").await.body;
    let untranslated = get_with_language(&app, "/posts/notes/letters", "es")
        .await
        .body;

    assert!(spanish.contains(&format!(r#"<link rel="canonical" href="{fog}?lang=es">"#)));
    assert!(spanish.contains(&format!(
        r#"<link rel="alternate" hreflang="en" href="{fog}?lang=en">"#
    )));
    assert!(spanish.contains(&format!(
        r#"<link rel="alternate" hreflang="es" href="{fog}?lang=es">"#
    )));
    assert!(spanish.contains(&format!(
        r#"<link rel="alternate" hreflang="x-default" href="{fog}">"#
    )));
    assert!(
        spanish
            .contains(r#"type="application/rss+xml" title="E2E garden" href="/rss.xml?lang=es""#)
    );
    assert!(!untranslated.contains(r#"rel="alternate" hreflang"#));
}

#[tokio::test]
async fn feeds_are_chosen_by_the_query_only() {
    let app = TestApp::spawn().await;

    let spanish = app.get("/rss.xml?lang=es").await.body;
    let default = get_with_language(&app, "/rss.xml", "es").await.body;

    assert!(spanish.contains("<language>es</language>"));
    assert!(spanish.contains("<title>Notas sobre la niebla</title>"));
    assert!(spanish.contains(&format!(
        "<link>{}/posts/notes/fog?lang=es</link>",
        app.address
    )));
    assert_eq!(spanish.matches("<item>").count(), 4);

    assert!(default.contains("<language>en</language>"));
    assert!(default.contains("<title>Notes about the fog</title>"));
}

#[tokio::test]
async fn the_layout_follows_the_language() {
    let app = TestApp::spawn().await;

    let spanish = get_with_language(&app, "/posts/notes/fog", "es").await.body;
    let english = get_with_language(&app, "/posts/notes/fog", "en").await.body;

    assert!(spanish.contains(r#"<html lang="es">"#));
    assert!(spanish.contains("INICIO"));
    assert!(spanish.contains(r#"placeholder="Busca cualquier post...""#));
    assert!(spanish.contains(r#"data-switcher-placeholder="Busca una nota...""#));
    assert!(spanish.contains(r#"aria-label="Panel lateral""#));
    assert!(spanish.contains(r#"lang="es" aria-current="true""#));
    assert!(!spanish.contains(">HOME<") && !spanish.contains("\n          HOME\n"));

    assert!(english.contains(r#"<html lang="en">"#));
    assert!(english.contains("HOME"));
    assert!(english.contains(r#"lang="en" aria-current="true""#));
}

#[tokio::test]
async fn the_language_switcher_reloads_the_whole_page() {
    let app = TestApp::spawn().await;

    let body = app.get("/posts/notes/fog").await.body;

    assert!(body.contains(r#"href="?lang=es" hx-boost="false""#));
    let chosen = app.get("/posts/notes/fog?lang=es").await;
    assert!(chosen.body.contains(r#"<html lang="es">"#));
    assert!(chosen.header("set-cookie").unwrap().starts_with("lang=es;"));
}

#[tokio::test]
async fn side_panels_and_listings_use_the_language() {
    let app = TestApp::spawn().await;
    let htmx_get = |path: &'static str| {
        let app = &app;
        async move {
            app.send(
                app.client
                    .get(app.url(path))
                    .header("HX-Request", "true")
                    .header("Accept-Language", "es"),
            )
            .await
            .body
        }
    };

    assert!(
        htmx_get("/metadata/notes/letters")
            .await
            .contains("Tiempo de lectura")
    );
    assert!(htmx_get("/tags").await.contains("ETIQUETAS"));
    assert!(
        htmx_get("/tags/gaming")
            .await
            .contains("← Todas las etiquetas")
    );
    let not_found = htmx_get("/posts/missing").await;
    assert!(not_found.contains("Esta página no existe (todavía)."));

    let news = htmx_get("/news").await;
    assert!(news.contains("Últimas novedades"));
    assert_eq!(news.matches("Notas sobre la niebla").count(), 1);
    assert!(
        !news.contains("Notes about the fog"),
        "translated posts are listed once"
    );
}

#[tokio::test]
async fn the_schedule_speaks_the_language() {
    let app = TestApp::spawn().await;
    let schedule = |lang: &'static str| {
        let app = &app;
        async move {
            app.send(
                app.client
                    .get(app.url("/schedule"))
                    .header("HX-Request", "true")
                    .header("Accept-Language", lang),
            )
            .await
            .body
        }
    };

    assert_ne!(schedule("es").await, schedule("en").await);
}

#[tokio::test]
async fn cached_pages_are_kept_per_language() {
    let app = TestApp::spawn().await;
    e2e::mocks::mount_steam(
        &app,
        serde_json::json!([{"appid": 10, "name": "Most Played", "playtime_forever": 6_000}]),
        1,
    )
    .await;
    let steam = |lang: &'static str| {
        let app = &app;
        async move {
            app.send(
                app.client
                    .get(app.url("/steam"))
                    .header("HX-Request", "true")
                    .header("Accept-Language", lang),
            )
            .await
            .body
        }
    };

    let english = steam("en").await;
    let spanish = steam("es").await;

    assert!(english.contains("Level 42") && english.contains("View on Steam"));
    assert!(spanish.contains("Nivel 42") && spanish.contains("Ver en Steam"));
}
