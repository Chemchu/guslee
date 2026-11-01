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
    assert!(!untranslated.contains("hreflang"));
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
