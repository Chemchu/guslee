use std::fs;
use std::path::Path;

use search_engine::PostsSearchEngine;
use search_engine::types::{Limit, Params};
use search_engine::utils::Post;
use tempfile::TempDir;

const REPO: &str = "https://github.com/user/repo/blob/main";

fn write_post(root: &Path, relative_path: &str, frontmatter: &str, body: &str) {
    let path = root.join(relative_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!("---\n{frontmatter}\n---\n{body}")).unwrap();
}

fn garden() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    write_post(
        root,
        "welcome.md",
        "title: \"Welcome to the greenhouse\"\ndate: \"2025-01-01\"\ndescription: \"Start here\"\ntags: [intro]",
        "Start with [the first note](notes/first_note).",
    );
    write_post(
        root,
        "notes/first_note.md",
        "title: \"First note about Silent Hill\"\ndate: \"2025-02-01\"\ndescription: \"Fog\"\ntopic: \"Notes\"\ntags: [gaming]",
        "The fog never lifts in this town.",
    );
    write_post(
        root,
        "notes/second_note.md",
        "title: \"Second note\"\ndate: \"2025-03-01\"\ndescription: \"More fog\"\ntopic: \"Notes\"\ntags: [gaming]",
        "James's letter mentions the fog again. Back to [welcome](welcome).",
    );
    write_post(
        root,
        "secret.md",
        "title: \"Secret silent plans\"\ndate: \"2025-04-01\"\ndescription: \"Hidden\"\ntags: [draft]\nis_draft: true",
        "Fog everywhere in the greenhouse. See [welcome](welcome).",
    );

    dir
}

async fn engine(garden: &TempDir) -> PostsSearchEngine {
    PostsSearchEngine::new_with_endpoint("mem://", REPO, garden.path().to_str().unwrap()).await
}

fn search(query: &str, limit: Option<usize>) -> Params {
    Params {
        query: Some(query.to_string()),
        limit: limit.map(Limit::Number),
    }
}

fn paths(posts: &[Post]) -> Vec<&str> {
    let mut paths: Vec<&str> = posts.iter().map(|p| p.file_path.as_str()).collect();
    paths.sort();
    paths
}

#[tokio::test]
async fn finds_posts_by_title() {
    let garden = garden();
    let engine = engine(&garden).await;

    let results = engine.query_posts(&search("silent", None)).await;

    assert_eq!(paths(&results), vec!["notes/first_note.md"]);
}

#[tokio::test]
async fn finds_posts_by_content() {
    let garden = garden();
    let engine = engine(&garden).await;

    let results = engine.query_posts(&search("fog", None)).await;

    assert_eq!(
        paths(&results),
        vec!["notes/first_note.md", "notes/second_note.md"]
    );
}

#[tokio::test]
async fn search_is_case_insensitive() {
    let garden = garden();
    let engine = engine(&garden).await;

    let results = engine.query_posts(&search("SILENT", None)).await;

    assert_eq!(paths(&results), vec!["notes/first_note.md"]);
}

#[tokio::test]
async fn drafts_never_appear_in_search_results() {
    let garden = garden();
    let engine = engine(&garden).await;

    for query in ["secret", "greenhouse", "fog", "silent"] {
        let results = engine.query_posts(&search(query, None)).await;
        assert!(
            !paths(&results).contains(&"secret.md"),
            "draft returned for {query:?}"
        );
    }
}

#[tokio::test]
async fn search_respects_the_limit() {
    let garden = garden();
    let engine = engine(&garden).await;

    let results = engine.query_posts(&search("fog", Some(1))).await;

    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn quotes_in_the_query_are_treated_as_text() {
    let garden = garden();
    let engine = engine(&garden).await;

    let results = engine.query_posts(&search("james's", None)).await;
    let injection = engine.query_posts(&search("x' OR true OR '", None)).await;

    assert!(results.len() <= 1);
    assert!(injection.is_empty());
}

#[tokio::test]
async fn get_post_returns_published_posts_only() {
    let garden = garden();
    let engine = engine(&garden).await;

    let post = engine.get_post("notes/first_note.md").await.unwrap();

    assert_eq!(post.metadata.title, "First note about Silent Hill");
    assert_eq!(post.metadata.topic.as_deref(), Some("Notes"));
    assert_eq!(post.file_name, "first_note.md");
    assert!(engine.get_post("secret.md").await.is_none());
    assert!(engine.get_post("missing.md").await.is_none());
}

#[tokio::test]
async fn overall_graph_links_posts_mentioning_each_other() {
    let garden = garden();
    let engine = engine(&garden).await;

    let graph = engine.get_overall_graph_data().await;

    let path_of = |id: usize| {
        graph
            .nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.file_path.as_str())
            .unwrap()
    };
    let mut edges: Vec<(&str, &str)> = graph
        .edges
        .iter()
        .map(|e| (path_of(e.source), path_of(e.target)))
        .collect();
    edges.sort();

    assert_eq!(
        edges,
        vec![
            ("notes/second_note.md", "welcome.md"),
            ("welcome.md", "notes/first_note.md"),
        ]
    );
}

#[tokio::test]
async fn overall_graph_leaves_drafts_out() {
    let garden = garden();
    let engine = engine(&garden).await;

    let graph = engine.get_overall_graph_data().await;

    let mut nodes: Vec<&str> = graph.nodes.iter().map(|n| n.file_path.as_str()).collect();
    nodes.sort();
    assert_eq!(
        nodes,
        vec!["notes/first_note.md", "notes/second_note.md", "welcome.md"]
    );
    assert_eq!(graph.edges.len(), 2);
}

#[tokio::test]
async fn related_graph_centres_on_the_requested_post() {
    let garden = garden();
    let engine = engine(&garden).await;

    let graph = engine.get_graph_from_related_posts("welcome.md").await;

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.nodes[0].id, 1);
    assert_eq!(graph.nodes[0].file_path, "welcome.md");
    assert_eq!(graph.nodes[1].file_path, "notes/first_note.md");
    assert_eq!(graph.edges.len(), 1);
    assert_eq!((graph.edges[0].source, graph.edges[0].target), (2, 1));
}

#[tokio::test]
async fn related_graph_is_empty_for_unknown_or_draft_posts() {
    let garden = garden();
    let engine = engine(&garden).await;

    for path in ["missing.md", "secret.md"] {
        let graph = engine.get_graph_from_related_posts(path).await;
        assert!(graph.nodes.is_empty() && graph.edges.is_empty());
    }
}

#[tokio::test]
async fn raw_query_returns_default_on_invalid_queries() {
    let garden = garden();
    let engine = engine(&garden).await;

    let posts: Vec<Post> = engine.raw_query("SELECT * FROM posts").await;
    let invalid: Vec<Post> = engine.raw_query("THIS IS NOT SURREALQL").await;

    assert_eq!(posts.len(), 4);
    assert!(invalid.is_empty());
}

#[tokio::test]
async fn in_memory_engines_are_isolated_from_each_other() {
    let garden_a = garden();
    let garden_b = tempfile::tempdir().unwrap();
    write_post(
        garden_b.path(),
        "only.md",
        "title: \"Only post\"\ndate: \"2025-01-01\"\ndescription: \"x\"\ntags: []",
        "",
    );

    let engine_a = engine(&garden_a).await;
    let engine_b = engine(&garden_b).await;

    let posts_a: Vec<Post> = engine_a.raw_query("SELECT * FROM posts").await;
    let posts_b: Vec<Post> = engine_b.raw_query("SELECT * FROM posts").await;

    assert_eq!(posts_a.len(), 4);
    assert_eq!(posts_b.len(), 1);
}
