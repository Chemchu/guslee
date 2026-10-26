use regex::Regex;
use serde::Deserialize;
use std::fs;
use surrealdb::Surreal;
use surrealdb::engine::any::Any;
use surrealdb::engine::any::connect;
use surrealdb::types::SurrealValue;
use walkdir::WalkDir;

use crate::types::EdgeFilePath;
use crate::types::Params;
use crate::types::{DEFAULT_SEARCH_LIMIT, GraphData, GraphEdge, GraphNode};
use crate::utils::{Post, extract_full_metadata};

pub mod types;
pub mod utils;

pub struct PostsSearchEngine {
    db: Surreal<Any>,
}

// Debug builds use a local SurrealDB server (handy to inspect the data), release builds an
// embedded in-memory database
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
        for entry in WalkDir::new(documents_path) {
            let entry = entry.expect("Error while accessing the WalkDir entry");
            let path = entry.path();
            if path.is_file()
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
                        let post = Post {
                            file_name,
                            file_path,
                            metadata: extract_full_metadata(repo_path, full_path.as_str()).unwrap(),
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

        let db = connect(db_endpoint).await.unwrap();

        db.use_ns("guslee").use_db("guslee").await.unwrap();
        db.query(
            "-- Start from a clean slate: in debug builds the DB is a long-lived local server
            REMOVE TABLE IF EXISTS posts;
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
            let mentioned_posts = get_mentioned_posts_in_post_content(inserted_post);
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

        PostsSearchEngine { db }
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

    pub async fn get_graph_from_related_posts(&self, main_node_file_path: &str) -> GraphData {
        let curr_post = self.get_post(main_node_file_path).await;

        if curr_post.is_none() {
            return GraphData::empty();
        }

        let query = "SELECT ->points_to->posts.* as related_posts FROM posts WHERE file_path = $file_path AND metadata.is_draft = false";

        let mut result = self
            .db
            .query(query)
            .bind(("file_path", main_node_file_path.to_string()))
            .await
            .unwrap();

        let related_posts: Vec<(String, String)> = result
            .take::<Option<QueryRelatedPostResult>>(0)
            .unwrap()
            .map(|r| r.related_posts)
            .unwrap_or_default()
            .iter()
            .map(|p| (p.metadata.title.clone(), p.file_path.clone()))
            .collect::<Vec<(String, String)>>();
        let main_node = GraphNode {
            id: 1,
            label: curr_post.as_ref().unwrap().metadata.title.to_string(),
            file_path: curr_post.unwrap().file_path.to_string(),
        };

        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();
        nodes.push(main_node);
        for (index, node) in related_posts.iter().enumerate() {
            nodes.push(GraphNode {
                id: index + 2,
                label: node.0.clone(),
                file_path: node.1.clone(),
            });
            edges.push(GraphEdge {
                source: index + 2,
                target: 1,
            });
        }

        GraphData { nodes, edges }
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

#[derive(SurrealValue, Deserialize)]
struct QueryRelatedPostResult {
    related_posts: Vec<Post>,
}

fn get_mentioned_posts_in_post_content(post: &Post) -> Vec<String> {
    let content = post.content.to_owned();
    let regex_pattern = Regex::new(r"\[([^\]]+)\]\(([^\)]+)\)").unwrap();

    let links: Vec<String> = regex_pattern
        .captures_iter(&content)
        .filter_map(|cap| {
            let link = &cap[2];
            if link.starts_with("http") || link.starts_with("https") {
                None
            } else {
                Some(format!("{}.md", link))
            }
        })
        .collect();

    links
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
            },
            content: content.to_string(),
        }
    }

    #[test]
    fn internal_markdown_links_become_post_paths() {
        let post = post_with_content("See [my intro](hello) and [notes](journal/2025/notes).");

        assert_eq!(
            get_mentioned_posts_in_post_content(&post),
            vec!["hello.md", "journal/2025/notes.md"]
        );
    }

    #[test]
    fn external_links_are_ignored() {
        let post =
            post_with_content("[Rust](https://rust-lang.org/) and [plain](http://example.com)");

        assert!(get_mentioned_posts_in_post_content(&post).is_empty());
    }
}
