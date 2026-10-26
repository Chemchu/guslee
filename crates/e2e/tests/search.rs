use e2e::TestApp;

fn result_titles(body: &str) -> Vec<String> {
    body.match_indices("title=\"")
        .map(|(start, _)| {
            let rest = &body[start + 7..];
            rest[..rest.find('"').unwrap()].to_string()
        })
        .collect()
}

async fn search(app: &TestApp, query: &str, limit: Option<usize>) -> Vec<String> {
    let mut request = app
        .client
        .get(app.url("/search"))
        .header("HX-Request", "true")
        .query(&[("query", query)]);
    if let Some(limit) = limit {
        request = request.query(&[("limit", limit)]);
    }
    let response = app.send(request).await;
    assert_eq!(response.status, 200);
    result_titles(&response.body)
}

#[tokio::test]
async fn search_only_answers_htmx_requests() {
    let app = TestApp::spawn().await;

    let response = app.get("/search?query=fog").await;

    assert_eq!(response.body, "Only HTMX requests for search engine");
}

#[tokio::test]
async fn finds_posts_by_title_and_content() {
    let app = TestApp::spawn().await;

    assert_eq!(search(&app, "hello", None).await, vec!["Hello fixture"]);

    let mut fog = search(&app, "fog", None).await;
    fog.sort();
    assert_eq!(
        fog,
        vec![
            "Hello fixture",
            "Letters &lt;and&gt; escaping",
            "Notes about the fog",
            "Welcome to the fixture garden"
        ],
        "hello embeds a fog section"
    );
}

#[tokio::test]
async fn drafts_are_never_found() {
    let app = TestApp::spawn().await;

    for query in ["secret", "silent", "nobody", "letters"] {
        let titles = search(&app, query, None).await;
        assert!(
            !titles.iter().any(|t| t.contains("Secret")),
            "draft found by {query:?}: {titles:?}"
        );
    }
}

#[tokio::test]
async fn quotes_in_queries_do_not_break_search() {
    let app = TestApp::spawn().await;

    let titles = search(&app, "james's", None).await;
    let injection = search(&app, "x' OR true OR '", None).await;

    assert!(titles.len() <= 1);
    assert!(injection.is_empty());
}

#[tokio::test]
async fn short_queries_list_every_published_post_with_pinned_ones_first() {
    let app = TestApp::spawn().await;

    let titles = search(&app, "*", None).await;

    assert_eq!(titles.len(), 4, "{titles:?}");
    assert_eq!(titles[0], "Hello fixture");
    assert!(titles.contains(&"Welcome to the fixture garden".to_string()));
    assert!(!titles.iter().any(|t| t.contains("Secret")));
}

#[tokio::test]
async fn posts_sharing_a_topic_are_grouped() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/search?query=*").await.body;

    assert!(body.contains(">Notes</summary>"));
    assert!(body.contains(">Introduction</summary>"));
}

#[tokio::test]
async fn limit_query_parameter_caps_the_results() {
    let app = TestApp::spawn().await;

    let all = search(&app, "fixture", None).await;
    let limited = search(&app, "fixture", Some(1)).await;

    assert!(all.len() >= 2, "{all:?}");
    assert_eq!(limited.len(), 1);
}

#[tokio::test]
async fn titles_are_escaped_in_results() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/search?query=*").await.body;

    assert!(body.contains("Letters &lt;and&gt; escaping"));
    assert!(!body.contains("Letters <and>"));
}
