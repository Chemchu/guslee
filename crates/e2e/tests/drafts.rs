use std::fs;
use std::path::Path;

use e2e::{GARDEN_PATH, TestApp};

const DRAFT_LINK: &str = "/posts/draft";
const DRAFT_TITLE: &str = "Secret silent draft";

fn assert_no_draft(what: &str, body: &str) {
    assert!(
        !body.contains(DRAFT_LINK) && !body.contains(DRAFT_TITLE),
        "{what} lists the draft:\n{body}"
    );
}

#[tokio::test]
async fn sidebar_never_lists_drafts() {
    let app = TestApp::spawn().await;

    for view in ["", "&view=topics", "&view=folders"] {
        for query in [
            "",
            "&query=secret",
            "&query=silent",
            "&query=draft",
            "&query=nobody",
        ] {
            let path = format!("/search?limit=100{view}{query}");
            assert_no_draft(&path, &app.htmx_get(&path).await.body);
        }
    }
}

#[tokio::test]
async fn quick_switcher_never_lists_drafts() {
    let app = TestApp::spawn().await;

    let body = app.get("/switcher.json").await.body;

    assert_no_draft("/switcher.json", &body);
    let posts: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        posts
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["path"] != "draft")
    );
}

#[tokio::test]
async fn other_listings_never_list_drafts() {
    let app = TestApp::spawn().await;

    for path in [
        "/news",
        "/tags",
        "/tags/draft",
        "/rss.xml",
        "/garden-view",
        "/metadata/welcome",
    ] {
        assert_no_draft(path, &app.htmx_get(path).await.body);
    }
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[tokio::test]
async fn new_posts_reach_the_switcher_and_sidebar_without_a_build_step() {
    let garden = tempfile::tempdir().unwrap();
    copy_dir(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/garden")
            .as_path(),
        garden.path(),
    );
    fs::write(
        garden.path().join("notes/brand_new.md"),
        "---\ntitle: \"Brand new note\"\ndate: \"2025-12-01\"\ndescription: \"x\"\ntags: []\n---\nHi.",
    )
    .unwrap();
    fs::write(
        garden.path().join("notes/new_draft.md"),
        "---\ntitle: \"Unfinished idea\"\ndate: \"2025-12-02\"\ndescription: \"x\"\ntags: []\nis_draft: true\n---\nWIP.",
    )
    .unwrap();
    let app = TestApp::spawn_with_garden(garden.path().to_str().unwrap()).await;

    let switcher = app.get("/switcher.json").await.body;
    let folders = app.htmx_get("/search?limit=100&view=folders").await.body;
    let topics = app.htmx_get("/search?limit=100&view=topics").await.body;

    for (what, body) in [
        ("switcher", &switcher),
        ("folders", &folders),
        ("topics", &topics),
    ] {
        assert!(
            body.contains("Brand new note"),
            "{what} misses the new post"
        );
        assert!(
            !body.contains("Unfinished idea"),
            "{what} lists the new draft"
        );
    }
    assert!(switcher.starts_with(r#"[{"title":"Brand new note","path":"notes/brand_new"}"#));
    assert_ne!(garden.path(), Path::new(GARDEN_PATH));
}
