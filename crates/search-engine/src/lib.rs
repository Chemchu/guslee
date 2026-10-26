use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::engine::any::connect;
use surrealdb::types::SurrealValue;
use walkdir::WalkDir;

use crate::links::{LinkIndex, WikiLink, find_wikilinks, markdown_link_target};
use crate::tags::{markdown_inline_tags, tag_matches};
use crate::types::EdgeFilePath;
use crate::types::Params;
use crate::types::{DEFAULT_SEARCH_LIMIT, GraphData, GraphEdge, GraphNode};
use crate::utils::{Post, extract_full_metadata, strip_frontmatter};

pub mod git_dates;
pub mod links;
pub mod tags;
pub mod types;
pub mod utils;

pub struct PostsSearchEngine {
    db: Surreal<Any>,
    link_index: LinkIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mention {
    pub file_path: String,
    pub title: String,
    pub before: String,
    pub matched: String,
    pub after: String,
}

#[derive(SurrealValue, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Backlink {
    pub file_path: String,
    pub title: String,
}

#[cfg(debug_assertions)]
pub const DEFAULT_DB_ENDPOINT: &str = "ws://127.0.0.1:8000";
#[cfg(not(debug_assertions))]
pub const DEFAULT_DB_ENDPOINT: &str = "mem://";

impl PostsSearchEngine {
    pub async fn new(repo_path: &str, documents_path: &str) -> PostsSearchEngine {
        Self::new_with_endpoint(DEFAULT_DB_ENDPOINT, repo_path, documents_path).await
    }

    pub async fn new_with_endpoint(
        db_endpoint: &str,
        repo_path: &str,
        documents_path: &str,
    ) -> PostsSearchEngine {
        let mut posts: Vec<Post> = Vec::new();
        let last_updated = git_dates::last_updated_dates(documents_path);
        let is_hidden = |entry: &walkdir::DirEntry| {
            entry.depth() > 0 && entry.file_name().to_string_lossy().starts_with('.')
        };
        for entry in WalkDir::new(documents_path)
            .into_iter()
            .filter_entry(|entry| !is_hidden(entry))
        {
            let entry = entry.expect("Error while accessing the WalkDir entry");
            let path = entry.path();
            if path.is_file()
                && path.extension().is_some_and(|ext| ext == "md")
                && let Ok(relative_path) = path.strip_prefix(documents_path)
            {
                let file_path = relative_path.to_string_lossy().to_string();
                let file_name = relative_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                let full_path = format!("{}/{}", documents_path, file_path);

                match fs::read_to_string(path) {
                    Ok(content) => {
                        let mut metadata =
                            extract_full_metadata(repo_path, full_path.as_str()).unwrap();
                        metadata.last_updated = last_updated.get(&file_path).cloned();
                        for tag in markdown_inline_tags(&content) {
                            if !metadata.tags.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
                                metadata.tags.push(tag);
                            }
                        }
                        let post = Post {
                            file_name,
                            file_path,
                            metadata,
                            content,
                        };
                        posts.push(post);
                    }
                    Err(e) => {
                        eprintln!("Failed to read file {}: {}", path.display(), e);
                    }
                }
            }
        }

        let link_index = LinkIndex::new(
            posts
                .iter()
                .filter(|post| !post.metadata.is_draft)
                .map(|post| &post.file_path),
        );

        let db = connect(db_endpoint).await.unwrap();

        db.use_ns("guslee").use_db("guslee").await.unwrap();
        db.query(
            "REMOVE TABLE IF EXISTS posts;
            REMOVE TABLE IF EXISTS points_to;

            DEFINE TABLE posts SCHEMAFULL;
            DEFINE FIELD file_name ON posts TYPE string;
            DEFINE FIELD file_path ON posts TYPE string;
            DEFINE FIELD metadata ON posts TYPE object;
            DEFINE FIELD metadata.title ON posts TYPE string;
            DEFINE FIELD metadata.description ON posts TYPE string;
            DEFINE FIELD metadata.tags ON posts TYPE array<string>;
            DEFINE FIELD metadata.date ON posts TYPE string;
            DEFINE FIELD metadata.topic ON posts TYPE option<string>;
            DEFINE FIELD metadata.post_source_url ON posts TYPE string;
            DEFINE FIELD metadata.is_draft ON posts TYPE bool DEFAULT false;
            DEFINE FIELD metadata.reading_time ON posts TYPE int;
            DEFINE FIELD metadata.last_updated ON posts TYPE option<string>;
            DEFINE FIELD content ON posts TYPE string;
            DEFINE INDEX file_path_index ON TABLE posts COLUMNS file_path UNIQUE;

            -- Define a custom analyzer
            DEFINE ANALYZER OVERWRITE full_text_analyzer TOKENIZERS class FILTERS lowercase, ascii, edgengram(2, 15);
            
            -- Create a full-text search index
            DEFINE INDEX ml_title ON TABLE posts FIELDS metadata.title FULLTEXT ANALYZER full_text_analyzer BM25 HIGHLIGHTS;
            DEFINE INDEX ml_content ON TABLE posts FIELDS content FULLTEXT ANALYZER full_text_analyzer BM25 HIGHLIGHTS;",
        )
        .await
        .and_then(|response| response.check())
        .expect("Failed to define the posts schema");
        let inserted_posts = db
            .insert::<Vec<Post>>("posts")
            .content(posts)
            .await
            .unwrap();

        for inserted_post in inserted_posts.iter() {
            let mentioned_posts = get_mentioned_posts_in_post_content(inserted_post, &link_index);
            if mentioned_posts.is_empty() {
                continue;
            }

            let source_path = inserted_post.file_path.clone();

            for mentioned_path in mentioned_posts {
                let query_string = "RELATE (SELECT id FROM posts WHERE file_path = $source)->points_to->(SELECT id FROM posts WHERE file_path = $target)";
                let _ = db
                    .query(query_string)
                    .bind(("source", source_path.clone()))
                    .bind(("target", mentioned_path))
                    .await
                    .unwrap();
            }
        }

        PostsSearchEngine { db, link_index }
    }

    pub fn link_index(&self) -> &LinkIndex {
        &self.link_index
    }

    pub async fn get_backlinks(&self, file_path: &str) -> Vec<Backlink> {
        let response = self
            .db
            .query(
                "SELECT in.file_path AS file_path, in.metadata.title AS title
                FROM points_to
                WHERE out.file_path = $path AND in.metadata.is_draft = false",
            )
            .bind(("path", file_path.to_string()))
            .await;

        let mut backlinks: Vec<Backlink> = match response.and_then(|mut r| r.take(0)) {
            Ok(backlinks) => backlinks,
            Err(e) => {
                eprintln!("Backlinks query failed: {}", e);
                Vec::new()
            }
        };
        backlinks.sort_by(|a, b| a.title.cmp(&b.title));
        backlinks
    }

    pub async fn query_posts(&self, params: &Params) -> Vec<Post> {
        let limit = match &params.limit {
            Some(l) => l.value(),
            None => DEFAULT_SEARCH_LIMIT.value(),
        };

        let query = params
            .query
            .as_ref()
            .map(|q| q.to_lowercase())
            .unwrap_or_default();
        let response = self
            .db
            .query(
                "SELECT *,
                    search::score(0) AS title_score,
                    search::score(1) AS content_score,
                    search::score(0) * 2 + search::score(1) AS combined_score
                FROM posts
                WHERE (metadata.title @0@ $query OR content @1@ $query)
                   AND metadata.is_draft = false
                ORDER BY combined_score DESC
                LIMIT $limit",
            )
            .bind(("query", query))
            .bind(("limit", limit))
            .await;

        match response.and_then(|mut r| r.take(0)) {
            Ok(posts) => posts,
            Err(e) => {
                eprintln!("Search query failed: {}", e);
                Vec::new()
            }
        }
    }

    pub async fn get_post(&self, file_path: &str) -> Option<Post> {
        self.db
            .query("SELECT * FROM posts WHERE file_path = $path AND metadata.is_draft = false")
            .bind(("path", file_path.to_string()))
            .await
            .unwrap()
            .take::<Option<Post>>(0)
            .unwrap()
    }

    pub async fn posts_with_tag(&self, tag: &str) -> Vec<Post> {
        self.published_posts()
            .await
            .into_iter()
            .filter(|post| post.metadata.tags.iter().any(|t| tag_matches(t, tag)))
            .collect()
    }

    pub async fn published_posts(&self) -> Vec<Post> {
        let mut posts: Vec<Post> = self
            .raw_query::<Vec<Post>>("SELECT * FROM posts WHERE metadata.is_draft = false")
            .await;
        posts.sort_by(|a, b| {
            b.metadata
                .date
                .cmp(&a.metadata.date)
                .then_with(|| a.metadata.title.cmp(&b.metadata.title))
        });
        posts
    }

    pub async fn tag_counts(&self) -> Vec<(String, usize)> {
        let posts = self.published_posts().await;
        let mut tags: Vec<String> = Vec::new();
        for tag in posts.iter().flat_map(|post| &post.metadata.tags) {
            let segments: Vec<&str> = tag.split('/').collect();
            for depth in 1..=segments.len() {
                let path = segments[..depth].join("/");
                if !tags.iter().any(|t| t.eq_ignore_ascii_case(&path)) {
                    tags.push(path);
                }
            }
        }
        tags.sort_by_key(|tag| {
            tag.split('/')
                .map(str::to_lowercase)
                .collect::<Vec<String>>()
        });

        tags.into_iter()
            .map(|tag| {
                let count = posts
                    .iter()
                    .filter(|post| post.metadata.tags.iter().any(|t| tag_matches(t, &tag)))
                    .count();
                (tag, count)
            })
            .collect()
    }

    pub async fn get_overall_graph_data(&self) -> GraphData {
        let mut posts_result = self
            .db
            .query("SELECT * FROM posts WHERE metadata.is_draft = false")
            .await
            .unwrap();
        let posts: Vec<Post> = posts_result.take(0).unwrap();

        let mut edges_result = self
            .db
            .query("SELECT in.file_path AS source, out.file_path AS target FROM points_to")
            .await
            .unwrap();

        let edges_raw: Vec<EdgeFilePath> = edges_result.take(0).unwrap_or_default();

        self.build_graph_data(posts, edges_raw).await
    }

    async fn build_graph_data(&self, posts: Vec<Post>, edges_raw: Vec<EdgeFilePath>) -> GraphData {
        let mut nodes = Vec::new();
        let mut id_map = std::collections::HashMap::new();

        for (index, post) in posts.iter().enumerate() {
            let id = index + 1;
            id_map.insert(post.file_path.clone(), id);

            nodes.push(GraphNode {
                id,
                label: post.metadata.title.clone(),
                file_path: post.file_path.clone(),
            });
        }

        let mut edges = Vec::new();
        for edge in edges_raw {
            if let (Some(&source), Some(&target)) =
                (id_map.get(&edge.source), id_map.get(&edge.target))
            {
                edges.push(GraphEdge { source, target });
            }
        }

        GraphData { nodes, edges }
    }

    pub async fn get_graph_from_related_posts(
        &self,
        main_node_file_path: &str,
        depth: usize,
    ) -> GraphData {
        if self.get_post(main_node_file_path).await.is_none() {
            return GraphData::empty();
        }

        let titles: HashMap<String, String> = self
            .published_posts()
            .await
            .into_iter()
            .map(|post| (post.file_path, post.metadata.title))
            .collect();
        let edges = self.published_edges().await;

        let mut paths = vec![main_node_file_path.to_string()];
        let mut frontier = paths.clone();
        for _ in 0..depth {
            let mut next = Vec::new();
            for path in &frontier {
                let outgoing = edges
                    .iter()
                    .filter(|e| &e.source == path)
                    .map(|e| &e.target);
                let incoming = edges
                    .iter()
                    .filter(|e| &e.target == path)
                    .map(|e| &e.source);
                for neighbour in outgoing.chain(incoming) {
                    if !paths.contains(neighbour) {
                        paths.push(neighbour.clone());
                        next.push(neighbour.clone());
                    }
                }
            }
            frontier = next;
        }

        let id_of = |path: &str| paths.iter().position(|p| p == path).map(|i| i + 1);
        let nodes = paths
            .iter()
            .enumerate()
            .map(|(index, path)| GraphNode {
                id: index + 1,
                label: titles.get(path).cloned().unwrap_or_default(),
                file_path: path.clone(),
            })
            .collect();
        let mut graph_edges: Vec<GraphEdge> = edges
            .iter()
            .filter_map(|edge| {
                Some(GraphEdge {
                    source: id_of(&edge.source)?,
                    target: id_of(&edge.target)?,
                })
            })
            .collect();
        graph_edges.sort_by_key(|e| (e.source, e.target));

        GraphData {
            nodes,
            edges: graph_edges,
        }
    }

    async fn published_edges(&self) -> Vec<EdgeFilePath> {
        let mut edges: Vec<EdgeFilePath> = self
            .raw_query(
                "SELECT in.file_path AS source, out.file_path AS target
                FROM points_to
                WHERE in.metadata.is_draft = false AND out.metadata.is_draft = false",
            )
            .await;
        edges.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));
        edges.dedup_by(|a, b| a.source == b.source && a.target == b.target);
        edges
    }

    pub async fn get_unlinked_mentions(&self, file_path: &str) -> Vec<Mention> {
        let Some(target) = self.get_post(file_path).await else {
            return Vec::new();
        };
        let title = target.metadata.title.trim();
        if title.chars().count() < 3 {
            return Vec::new();
        }
        let Ok(pattern) = Regex::new(&format!(r"(?i)(?:^|\W)({})(?:\W|$)", regex::escape(title)))
        else {
            return Vec::new();
        };
        let linked: HashSet<String> = self
            .get_backlinks(file_path)
            .await
            .into_iter()
            .map(|b| b.file_path)
            .collect();

        let mut mentions: Vec<Mention> = self
            .published_posts()
            .await
            .into_iter()
            .filter(|post| post.file_path != file_path && !linked.contains(&post.file_path))
            .filter_map(|post| {
                let body = strip_frontmatter(&post.content);
                let found = pattern.captures(body)?.get(1)?;
                Some(Mention {
                    file_path: post.file_path,
                    title: post.metadata.title,
                    before: snippet_before(&body[..found.start()]),
                    matched: found.as_str().to_string(),
                    after: snippet_after(&body[found.end()..]),
                })
            })
            .collect();
        mentions.sort_by(|a, b| a.title.cmp(&b.title));
        mentions
    }

    /* pub async fn raw_query<T>(&self, query: &str) -> T
    where
        T: for<'de> serde::Deserialize<'de> + Default,
        usize: surrealdb::opt::QueryResult<T>,
    {
        match self.db.query(query).await {
            Ok(mut r) => r.take::<T>(0).unwrap_or_default(),
            Err(_) => T::default(),
        }
    } */

    pub async fn raw_query<T>(&self, query: &str) -> T
    where
        T: for<'de> serde::Deserialize<'de> + Default + SurrealValue,
        usize: surrealdb::opt::QueryResult<T>,
    {
        match self.db.query(query).await {
            Ok(mut r) => r.take::<T>(0).unwrap_or_default(),
            Err(_) => T::default(),
        }
    }
}

const SNIPPET_CHARS: usize = 40;

fn snippet_before(text: &str) -> String {
    let text = collapse_whitespace(text);
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= SNIPPET_CHARS {
        return text;
    }
    let tail: String = chars[chars.len() - SNIPPET_CHARS..].iter().collect();
    let tail = tail.split_once(' ').map_or(tail.as_str(), |(_, rest)| rest);
    format!("…{tail}")
}

fn snippet_after(text: &str) -> String {
    let text = collapse_whitespace(text);
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= SNIPPET_CHARS {
        return text;
    }
    let head: String = chars[..SNIPPET_CHARS].iter().collect();
    let head = head
        .rsplit_once(' ')
        .map_or(head.as_str(), |(start, _)| start);
    format!("{head}…")
}

fn collapse_whitespace(text: &str) -> String {
    let mut collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.starts_with(char::is_whitespace) && !collapsed.is_empty() {
        collapsed.insert(0, ' ');
    }
    if text.ends_with(char::is_whitespace) && !collapsed.is_empty() {
        collapsed.push(' ');
    }
    collapsed
}

fn get_mentioned_posts_in_post_content(post: &Post, link_index: &LinkIndex) -> Vec<String> {
    let regex_pattern = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();

    let markdown_links = regex_pattern
        .captures_iter(&post.content)
        .filter_map(|cap| markdown_link_target(&cap[2]))
        .filter(|target| link_index.contains(&target.path))
        .map(|target| target.file_path());
    let wikilinks = find_wikilinks(&post.content)
        .into_iter()
        .filter_map(|m| link_index.wikilink_target(&WikiLink::parse(m.inner)))
        .map(|target| target.file_path());

    let mut seen = HashSet::new();
    markdown_links
        .chain(wikilinks)
        .filter(|file_path| *file_path != post.file_path && seen.insert(file_path.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MdMetadata;

    fn post_with_content(content: &str) -> Post {
        Post {
            file_name: "post.md".to_string(),
            file_path: "post.md".to_string(),
            metadata: MdMetadata {
                title: "Post".to_string(),
                topic: None,
                description: String::new(),
                tags: vec![],
                date: String::new(),
                is_draft: false,
                post_source_url: String::new(),
                reading_time: 0,
                last_updated: None,
            },
            content: content.to_string(),
        }
    }

    fn index() -> LinkIndex {
        LinkIndex::new([
            "hello.md",
            "post.md",
            "notes/fog.md",
            "journal/2025/notes.md",
        ])
    }

    #[test]
    fn internal_markdown_links_become_post_paths() {
        let post = post_with_content("See [my intro](hello) and [notes](journal/2025/notes).");

        assert_eq!(
            get_mentioned_posts_in_post_content(&post, &index()),
            vec!["hello.md", "journal/2025/notes.md"]
        );
    }

    #[test]
    fn markdown_links_with_anchors_or_extensions_are_found() {
        let post = post_with_content("[a](notes/fog#the-fog), [b](./hello.md) and [c](hello)");

        assert_eq!(
            get_mentioned_posts_in_post_content(&post, &index()),
            vec!["notes/fog.md", "hello.md"]
        );
    }

    #[test]
    fn wikilinks_are_found_by_note_name() {
        let post = post_with_content("[[fog#The fog|that fog]], [[Hello]] and [[missing]]");

        assert_eq!(
            get_mentioned_posts_in_post_content(&post, &index()),
            vec!["notes/fog.md", "hello.md"]
        );
    }

    #[test]
    fn self_links_and_missing_posts_are_not_links() {
        let post = post_with_content("[[post]] [me](post) [gone](gone) ![img](pic.png)");

        assert!(get_mentioned_posts_in_post_content(&post, &index()).is_empty());
    }

    #[test]
    fn embeds_count_as_links() {
        let post = post_with_content("![[fog#The fog]]");

        assert_eq!(
            get_mentioned_posts_in_post_content(&post, &index()),
            vec!["notes/fog.md"]
        );
    }

    #[test]
    fn snippets_are_cut_at_words_around_the_mention() {
        assert_eq!(snippet_before("short\n text "), "short text ");
        assert_eq!(
            snippet_before("one two three four five six seven eight nine ten "),
            "…four five six seven eight nine ten "
        );
        assert_eq!(
            snippet_after(" one two three four five six seven eight nine ten"),
            " one two three four five six seven…"
        );
    }

    #[test]
    fn external_links_are_ignored() {
        let post =
            post_with_content("[Rust](https://rust-lang.org/) and [plain](http://example.com)");

        assert!(get_mentioned_posts_in_post_content(&post, &index()).is_empty());
    }
}
