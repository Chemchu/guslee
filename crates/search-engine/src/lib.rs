use std::fs;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::{Response, Surreal};
use walkdir::WalkDir;

use crate::types::{DEFAULT_SEARCH_LIMIT, SearchResult};
use crate::types::{MatchingFile, Params};
use crate::utils::{Post, extract_full_metadata};

pub mod types;
pub mod utils;

pub struct SearchEngine {
    db: Surreal<Db>,
    default_results: Vec<String>,
}

impl SearchEngine {
    pub async fn new(documents_path: &str, default_docs: Vec<String>) -> SearchEngine {
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
        let db = Surreal::new::<Mem>(()).await.unwrap();
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
            DEFINE FIELD content ON posts TYPE string;
            DEFINE INDEX file_path_index ON TABLE posts COLUMNS file_path UNIQUE;

            -- Define a custom analyzer
            DEFINE ANALYZER full_text_analyzer TOKENIZERS class FILTERS lowercase, ascii, edgengram(2, 15);
            
            -- Create a full-text search index
            DEFINE INDEX ml_title ON TABLE posts FIELDS metadata.title SEARCH ANALYZER full_text_analyzer BM25 HIGHLIGHTS;
            DEFINE INDEX ml_content ON TABLE posts FIELDS content SEARCH ANALYZER full_text_analyzer BM25 HIGHLIGHTS;",
        )
        .await;
        let _ = db.insert::<Vec<Post>>("posts").content(posts).await;

        SearchEngine {
            db,
            default_results: default_docs,
        }
    }

    pub async fn exec_query(&self, params: &Params) -> SearchResult {
        let limit = match &params.limit {
            Some(l) => l.value(),
            None => DEFAULT_SEARCH_LIMIT.value(),
        };

        let is_empty_query = params.query.is_none()
            || params.query.as_ref().unwrap().is_empty()
            || params.query.as_ref().unwrap() == "*"
            || params.query.as_ref().unwrap().len() < 3;

        if is_empty_query {
            return self.query_default_docs().await;
        }

        let query = params.query.as_ref().unwrap();
        self.exec_query_internal(query.to_lowercase().as_str(), limit)
            .await
    }

    async fn exec_query_internal(&self, query: &str, result_limit: usize) -> SearchResult {
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
                query, query, result_limit
            ))
            .await
            .unwrap();

        let docs: Vec<Post> = response.take(0).unwrap();

        SearchResult {
            matching_files: docs
                .iter()
                .map(|post| MatchingFile::new(post.metadata.title.clone(), post.file_path.clone()))
                .collect(),
        }
    }

    async fn query_default_docs(&self) -> SearchResult {
        let default_docs_display_string = self
            .default_results
            .iter()
            .map(|f| format!("'{}'", f))
            .collect::<Vec<String>>()
            .join(", ");

        let files: Vec<MatchingFile> = self
            .db
            .query(format!(
                "SELECT * FROM posts WHERE file_name IN [{}]",
                default_docs_display_string
            ))
            .await
            .unwrap()
            .take::<Vec<Post>>(0)
            .unwrap()
            .iter()
            .map(|post| (post.metadata.title.clone(), post.file_name.clone()))
            .map(|(file_title, file_path)| MatchingFile::new(file_title, file_path.to_string()))
            .collect();

        SearchResult {
            matching_files: files,
        }
    }
}
