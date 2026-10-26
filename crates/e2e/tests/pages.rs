use e2e::{APP_NAME, REPO_SOURCE, TestApp};

#[tokio::test]
async fn landing_page_renders_the_welcome_post_in_the_layout() {
    let app = TestApp::spawn().await;

    let response = app.get("/").await;

    assert_eq!(response.status, 200);
    assert!(response.is_full_page());
    assert!(
        response
            .body
            .contains(&format!("<title>{APP_NAME}</title>"))
    );
    assert!(response.body.contains("This garden only exists for tests."));
}

#[tokio::test]
async fn layout_is_wired_for_htmx_4() {
    let app = TestApp::spawn().await;

    let body = app.get("/").await.body;

    assert!(body.contains("hx-boost:inherited=\"true\""));
    let config = body.find("name=\"htmx-config\"").unwrap();
    let script = body.find("/_static/htmx.min.js").unwrap();
    assert!(config < script);
    assert!(body.contains(r#"hx-vals='{"limit": 100}'"#));
}

#[tokio::test]
async fn links_inside_the_content_only_swap_the_main_section() {
    let app = TestApp::spawn().await;

    let body = app.get("/posts/welcome").await.body;
    let main_start = body.find("id=\"main-section\"").unwrap();
    let main_tag = &body[main_start..main_start + body[main_start..].find('>').unwrap()];

    assert!(main_tag.contains("hx-target:inherited=\"#main-section\""));
    assert!(main_tag.contains("hx-swap:inherited=\"innerHTML transition:true\""));
    assert!(main_tag.contains("hx-history-elt"));
    assert!(body[main_start..].contains("href=\"/posts/notes/fog\""));
}

#[tokio::test]
async fn links_in_nested_posts_resolve_from_the_garden_root() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/posts/notes/letters").await.body;

    assert!(body.contains("<a href=\"/posts/notes/fog\">the fog note</a>"));
    assert_eq!(app.htmx_get("/posts/notes/fog").await.status, 200);
}

#[tokio::test]
async fn posts_are_full_pages_for_browsers_and_fragments_for_htmx() {
    let app = TestApp::spawn().await;

    let full = app.get("/posts/notes/fog").await;
    let partial = app.htmx_get("/posts/notes/fog").await;

    assert_eq!(full.status, 200);
    assert!(full.is_full_page());
    assert_eq!(partial.status, 200);
    assert!(!partial.is_full_page());
    assert!(!partial.body.contains("<html"));
    for body in [&full.body, &partial.body] {
        assert!(body.contains(r#"<h1 id="the-fog">The fog</h1>"#));
        assert!(body.contains("<strong>fog</strong>"));
        assert!(
            !body.contains("topic: \"Notes\""),
            "frontmatter must not render"
        );
    }
}

#[tokio::test]
async fn back_button_restores_get_the_full_page() {
    let app = TestApp::spawn().await;

    for path in ["/posts/notes/fog", "/news", "/garden-view-dispatcher"] {
        let response = app.history_restore_get(path).await;
        assert!(response.is_full_page(), "{path} must render the layout");
    }
}

#[tokio::test]
async fn post_page_loads_its_graph_and_metadata_lazily() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/posts/notes/fog").await.body;

    assert!(body.contains("hx-get=\"/graph/notes/fog\""));
    assert!(body.contains("hx-get=\"/metadata/notes/fog\""));
}

#[tokio::test]
async fn missing_posts_and_routes_are_404s() {
    let app = TestApp::spawn().await;

    let missing_post = app.get("/posts/does/not/exist").await;
    let missing_post_partial = app.htmx_get("/posts/does/not/exist").await;
    let unknown_route = app.get("/totally/unknown").await;

    assert_eq!(missing_post.status, 404);
    assert!(missing_post.is_full_page());
    assert!(
        missing_post
            .body
            .contains("This page does not exist (yet).")
    );
    assert_eq!(missing_post_partial.status, 404);
    assert!(!missing_post_partial.is_full_page());
    assert_eq!(unknown_route.status, 404);
    assert!(unknown_route.is_full_page());
}

#[tokio::test]
async fn post_routes_cannot_escape_the_garden() {
    let app = TestApp::spawn().await;

    let (status, body) = app.raw_get("/posts/../../../../README").await;

    assert_eq!(status, 404);
    assert!(!body.contains("My digital garden project"));
}

#[tokio::test]
async fn news_lists_published_posts_newest_first() {
    let app = TestApp::spawn().await;

    let response = app.htmx_get("/news").await;

    assert_eq!(response.status, 200);
    let body = &response.body;
    assert!(
        !body.contains("Secret silent draft"),
        "drafts must not be listed"
    );
    let order: Vec<usize> = [
        "Letters &lt;and&gt; escaping",
        "Notes about the fog",
        "Hello fixture",
        "Welcome to the fixture garden",
    ]
    .iter()
    .map(|title| {
        body.find(title)
            .unwrap_or_else(|| panic!("{title} missing"))
    })
    .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "newest first");
}

#[tokio::test]
async fn garden_view_dispatcher_lazy_loads_the_graph() {
    let app = TestApp::spawn().await;

    let partial = app.htmx_get("/garden-view-dispatcher").await;
    let full = app.get("/garden-view-dispatcher").await;

    assert!(!partial.is_full_page());
    assert!(partial.body.contains("hx-get=\"/garden-view\""));
    assert!(full.is_full_page());
}

fn data_attribute(body: &str, name: &str) -> serde_json::Value {
    let start = body.find(&format!("{name}=\"")).unwrap() + name.len() + 2;
    let end = start + body[start..].find('"').unwrap();
    let json = body[start..end].replace("&quot;", "\"");
    serde_json::from_str(&json).unwrap()
}

#[tokio::test]
async fn garden_graph_contains_published_posts_and_their_links() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/garden-view").await.body;
    let nodes = data_attribute(&body, "data-nodes");
    let edges = data_attribute(&body, "data-edges");

    let mut paths: Vec<&str> = nodes
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["file_path"].as_str().unwrap())
        .collect();
    paths.sort();
    assert_eq!(
        paths,
        vec!["hello.md", "notes/fog.md", "notes/letters.md", "welcome.md"]
    );
    assert_eq!(edges.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn post_graph_shows_its_links_and_backlinks() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/graph/welcome").await.body;
    let nodes = data_attribute(&body, "data-nodes");

    let labels: Vec<&str> = nodes
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["label"].as_str().unwrap())
        .collect();
    assert_eq!(
        labels,
        vec![
            "Welcome to the fixture garden",
            "Notes about the fog",
            "Hello fixture"
        ]
    );
}

#[tokio::test]
async fn wikilinks_render_as_links_or_unresolved_notes() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/posts/notes/letters").await.body;

    assert!(body.contains(r#"<a href="/posts/notes/fog#the-fog">the fog itself</a>"#));
    assert!(body.contains(r#"<span class="wikilink-unresolved""#));
    assert!(body.contains(">ghosts</span>"));
}

#[tokio::test]
async fn metadata_panel_lists_linked_mentions() {
    let app = TestApp::spawn().await;

    let fog = app.htmx_get("/metadata/notes/fog").await.body;
    let welcome = app.htmx_get("/metadata/welcome").await.body;
    let hello = app.htmx_get("/metadata/hello").await.body;

    assert!(fog.contains("Linked mentions"));
    let letters = fog.find("href=\"/posts/notes/letters\"").unwrap();
    let welcome_link = fog.find("href=\"/posts/welcome\"").unwrap();
    assert!(letters < welcome_link, "sorted by title");
    assert!(welcome.contains("href=\"/posts/hello\""));
    assert!(!welcome.contains("/posts/draft"), "drafts are not mentions");
    assert!(hello.contains("No notes link here yet."));
}

#[tokio::test]
async fn metadata_panel_shows_frontmatter_and_source_link() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/metadata/notes/fog").await.body;
    let draft = app.htmx_get("/metadata/draft").await.body;

    assert!(body.contains("Notes about the fog"));
    assert!(body.contains("Everything about the fog"));
    assert!(body.contains(&format!(
        "{REPO_SOURCE}/crates/e2e/fixtures/garden/notes/fog.md"
    )));
    assert!(draft.contains("No metadata found for this post."));
}

#[tokio::test]
async fn schedule_returns_an_activity() {
    let app = TestApp::spawn().await;

    let response = app.htmx_get("/schedule").await;

    assert_eq!(response.status, 200);
    assert!(response.body.starts_with("<span class=\"italic\">"));
}

#[tokio::test]
async fn static_files_are_served_compressed_without_a_directory_listing() {
    let app = TestApp::spawn().await;

    let htmx = app
        .send(
            app.client
                .get(app.url("/_static/htmx.min.js"))
                .header("Accept-Encoding", "br"),
        )
        .await;
    let listing = app.get("/_static/").await;

    assert_eq!(htmx.status, 200);
    assert_eq!(htmx.header("content-encoding"), Some("br"));
    assert!(htmx.header("content-type").unwrap().contains("javascript"));
    assert_eq!(listing.status, 404);
}

#[tokio::test]
async fn vendored_htmx_is_version_4() {
    let app = TestApp::spawn().await;

    let htmx = app.get("/_static/htmx.min.js").await;

    assert!(htmx.body.contains("\"4.0.0\""));
}

#[tokio::test]
async fn sidebar_has_a_drag_handle_to_resize_it() {
    let app = TestApp::spawn().await;

    let body = app.get("/").await.body;
    let sidebar_start = body.find("id=\"left-section\"").unwrap();
    let sidebar_tag =
        &body[sidebar_start..sidebar_start + body[sidebar_start..].find('>').unwrap()];

    assert!(sidebar_tag.contains("md:w-[var(--sidebar-width,18rem)]"));
    assert!(!sidebar_tag.contains("border-r"));
    assert!(body.contains("id=\"sidebar-resizer\""));
    assert!(body.contains("/_static/resizable.js"));
    assert_eq!(app.get("/_static/resizable.js").await.status, 200);
}

#[tokio::test]
async fn post_side_panel_has_a_drag_handle_to_resize_it() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/posts/notes/fog").await.body;
    let panel_start = body.find("id=\"right-section\"").unwrap();
    let panel_tag = &body[panel_start..panel_start + body[panel_start..].find('>').unwrap()];
    let handle = body.find("id=\"right-section-resizer\"").unwrap();

    assert!(handle < panel_start);
    assert!(body[handle..panel_start].contains("data-resize-target=\"right-section\""));
    assert!(body[handle..panel_start].contains("data-resize-edge=\"left\""));
    assert!(panel_tag.contains("lg:w-[var(--right-section-width,20rem)]"));
    assert!(!panel_tag.contains("border-l"));
}

#[tokio::test]
async fn headings_have_anchors_that_heading_links_point_to() {
    let app = TestApp::spawn().await;

    let fog = app.htmx_get("/posts/notes/fog").await.body;
    let letters = app.htmx_get("/posts/notes/letters").await.body;

    assert!(fog.contains(r#"<h1 id="the-fog">The fog</h1>"#));
    assert!(letters.contains(r#"href="/posts/notes/fog#the-fog""#));
}

#[tokio::test]
async fn hovering_a_link_previews_the_post() {
    let app = TestApp::spawn().await;

    let preview = app.get("/preview/notes/fog").await;

    assert_eq!(preview.status, 200);
    assert!(!preview.is_full_page());
    assert!(
        preview
            .body
            .contains(r#"<h1 data-preview-anchor="the-fog">The fog</h1>"#)
    );
    assert!(preview.body.contains("never lifts in Silent Hill"));
    assert_eq!(app.get("/preview/draft").await.status, 404);
    assert_eq!(app.get("/preview/missing").await.status, 404);

    let layout = app.get("/").await.body;
    assert!(layout.contains("/_static/preview.js"));
    assert_eq!(app.get("/_static/preview.js").await.status, 200);
}

#[tokio::test]
async fn metadata_tags_link_to_their_tag_page() {
    let app = TestApp::spawn().await;

    let body = app.htmx_get("/metadata/welcome").await.body;

    assert!(body.contains(r#"href="/tags/intro""#));
    assert!(body.contains(r#"href="/tags/digital%20garden""#));
}

#[tokio::test]
async fn tag_pages_list_published_posts_with_that_tag() {
    let app = TestApp::spawn().await;

    let intro = app.htmx_get("/tags/intro").await;
    let spaced = app.htmx_get("/tags/digital%20garden").await;
    let full = app.get("/tags/INTRO").await;

    assert_eq!(intro.status, 200);
    assert!(intro.body.contains("#intro"));
    assert!(intro.body.contains("2 notes"));
    let hello = intro.body.find(r#"href="/posts/hello""#).unwrap();
    let welcome = intro.body.find(r#"href="/posts/welcome""#).unwrap();
    assert!(hello < welcome, "newest first");
    assert!(!intro.body.contains("/posts/draft"));
    assert!(spaced.body.contains("1 note<"));
    assert!(full.is_full_page() && full.body.contains("2 notes"));
}

#[tokio::test]
async fn tags_only_used_by_drafts_or_unknown_are_404s() {
    let app = TestApp::spawn().await;

    assert_eq!(app.htmx_get("/tags/draft").await.status, 404);
    assert_eq!(app.htmx_get("/tags/nothing-here").await.status, 404);
}
