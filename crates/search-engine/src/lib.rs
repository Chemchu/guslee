use regex::Regex;
use std::collections::HashMap;
use std::fs;
use surrealdb::engine::any::Any;
#[cfg(debug_assertions)]
use surrealdb::engine::any::connect;
use surrealdb::{Response, Surreal};
use walkdir::WalkDir;

use crate::types::{DEFAULT_SEARCH_LIMIT, GraphData, SearchResult};
use crate::types::{MatchingFile, Params};
use crate::utils::{Post, extract_full_metadata};

pub mod types;
pub mod utils;

pub struct SearchEngine {
    db: Surreal<Any>,
}

impl SearchEngine {
    pub async fn new(documents_path: &str) -> SearchEngine {
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
                let full_path = format!("{}/{}", documents_path, &file_path);

                match fs::read_to_string(path) {
                    Ok(content) => {
                        let post = Post {
                            file_name,
                            file_path,
                            metadata: extract_full_metadata(full_path.as_str()).unwrap(),
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

        #[cfg(debug_assertions)]
        let db = connect("ws://127.0.0.1:8000").await.unwrap();

        #[cfg(not(debug_assertions))]
        let db = connect("mem://").await.unwrap();

        db.use_ns("guslee").use_db("guslee").await.unwrap();
        let _  = db.query(
            "DEFINE TABLE posts SCHEMAFULL;
            DEFINE FIELD file_name ON posts TYPE string;
            DEFINE FIELD file_path ON posts TYPE string;
            DEFINE FIELD metadata ON posts TYPE object;
            DEFINE FIELD metadata.title ON posts TYPE string;
            DEFINE FIELD metadata.description ON posts TYPE string;
            DEFINE FIELD metadata.tags ON posts TYPE array<string>;
            DEFINE FIELD metadata.date ON posts TYPE string;
            DEFINE FIELD metadata.topic ON posts TYPE option<string>;
            DEFINE FIELD content ON posts TYPE string;
            DEFINE INDEX file_path_index ON TABLE posts COLUMNS file_path UNIQUE;

            -- Define a custom analyzer
            DEFINE ANALYZER full_text_analyzer TOKENIZERS class FILTERS lowercase, ascii, edgengram(2, 15);
            
            -- Create a full-text search index
            DEFINE INDEX ml_title ON TABLE posts FIELDS metadata.title SEARCH ANALYZER full_text_analyzer BM25 HIGHLIGHTS;
            DEFINE INDEX ml_content ON TABLE posts FIELDS content SEARCH ANALYZER full_text_analyzer BM25 HIGHLIGHTS;",
        )
        .await;
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
                    .bind(("source", source_path.clone())) // Pass owned String, not &str
                    .bind(("target", mentioned_path)) // This is already owned
                    .await
                    .unwrap();
            }
        }

        // Query graph examples
        //select ->points_to->posts.file_path AS related_posts from posts:uoksb5bh4hwasv99frxy;
        //select ->points_to->posts.file_path AS related_posts from posts;

        SearchEngine { db }
    }

    pub async fn query_posts(&self, params: &Params) -> SearchResult {
        let limit = match &params.limit {
            Some(l) => l.value(),
            None => DEFAULT_SEARCH_LIMIT.value(),
        };

        let query = params.query.as_ref().unwrap().to_lowercase();
        let mut response: Response = self
            .db
            .query(format!(
                "SELECT *,
                    search::score(0) AS title_score,
                    search::score(1) AS content_score,
                    search::score(0) * 2 + search::score(1) AS combined_score
                FROM posts 
                WHERE metadata.title @0@ '{}' 
                   OR content @1@ '{}'
                ORDER BY combined_score DESC
                LIMIT {}",
                query, query, limit
            ))
            .await
            .unwrap();

        let docs: Vec<Post> = response.take(0).unwrap();

        SearchResult {
            matching_files: docs
                .iter()
                .map(|post| {
                    MatchingFile::new(
                        post.metadata.title.clone(),
                        post.file_name.clone(),
                        post.file_path.clone(),
                        post.metadata.topic.clone(),
                    )
                })
                .collect(),
        }
    }

    pub async fn query_from_list(&self, posts_to_search: Vec<&str>) -> SearchResult {
        let default_docs_display_string = posts_to_search
            .iter()
            .map(|file_name| format!("'{}'", file_name))
            .collect::<Vec<String>>()
            .join(", ");

        let files: Vec<MatchingFile> = self
            .db
            .query(format!(
                "SELECT * FROM posts WHERE file_name IN [{}]",
                default_docs_display_string,
            ))
            .await
            .unwrap()
            .take::<Vec<Post>>(0)
            .unwrap()
            .iter()
            .map(|post| {
                MatchingFile::new(
                    post.metadata.title.clone(),
                    post.file_name.clone(),
                    post.file_path.to_string(),
                    post.metadata.topic.clone(),
                )
            })
            .collect();

        let map: HashMap<String, MatchingFile> = files
            .into_iter()
            .map(|f| (f.file_name().to_string(), f))
            .collect();

        let matching_files: Vec<MatchingFile> = posts_to_search
            .iter()
            .filter_map(|default_doc| map.get(*default_doc).cloned())
            .collect();

        SearchResult { matching_files }
    }

    pub async fn get_post(&self, file_path: &str) -> MatchingFile {
        let post: Post = self
            .db
            .query("SELECT * FROM posts WHERE file_path = $path")
            .bind(("path", file_path.to_string()))
            .await
            .unwrap()
            .take::<Option<Post>>(0)
            .unwrap()
            .unwrap();

        MatchingFile::new(
            post.metadata.title.clone(),
            post.file_name.clone(),
            post.file_path.to_string(),
            post.metadata.topic.clone(),
        )
    }

    pub fn get_related_posts(file_path: &str) -> GraphData {
        todo!();
    }
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
