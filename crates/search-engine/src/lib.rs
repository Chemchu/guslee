use lru::LruCache;
use serde::Deserialize;
use std::sync::Arc;
use std::{fs, num::NonZeroUsize, sync::Mutex};
use tantivy::query::{BooleanQuery, Occur};
use tantivy::{
    Index, IndexReader, IndexWriter, TantivyDocument,
    query::RegexQuery,
    schema::{Field, STORED, Schema, TEXT, Value},
};
use tantivy_fst::Regex;
use walkdir::WalkDir;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Limit {
    Number(usize),
    String(String),
}

#[derive(Deserialize, Clone)]
pub struct Params {
    pub query: Option<String>,
    pub limit: Option<Limit>,
}

const DEFAULT_SEARCH_LIMIT: Limit = Limit::Number(100);

impl Limit {
    pub fn value(&self) -> usize {
        match self {
            Limit::Number(val) => *val,
            Limit::String(_val) => DEFAULT_SEARCH_LIMIT.value(),
        }
    }
}

#[derive(Clone)]
pub struct SearchResult {
    pub matching_files: Vec<MatchingFile>,
}

#[derive(Clone)]
pub struct MatchingFile {
    name: String,
    path: String,
}

impl MatchingFile {
    pub fn file_name(&self) -> &str {
        &self.name
    }

    pub fn file_path(&self) -> &str {
        &self.path
    }
}

pub struct SearchEngine {
    reader: IndexReader,
    index: Index,
    lru_cache: Mutex<LruCache<String, SearchResult>>,
    default_results: Vec<String>,
}

impl SearchEngine {
    pub fn new(documents_path: &str, default_docs: Vec<String>) -> SearchEngine {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();

        // Use in-memory index to avoid file system issues
        let index = Index::create_in_ram(schema.clone());

        let mut index_writer: IndexWriter = index
            .writer(50_000_000)
            .expect("Failed to allocate 50MB to index");

        let title: Field = schema.get_field("title").unwrap();
        let body: Field = schema.get_field("body").unwrap();

        // Index documents
        for entry in WalkDir::new(documents_path) {
            let entry = entry.expect("Error while accessing the WalkDir entry");
            let path = entry.path();
            if path.is_file()
                && let Ok(relative_path) = path.strip_prefix(documents_path)
            {
                let file_path = relative_path.to_string_lossy().to_string();

                match fs::read_to_string(path) {
                    Ok(content) => {
                        let mut doc = TantivyDocument::default();
                        doc.add_text(title, file_path);
                        doc.add_text(body, &content);
                        index_writer
                            .add_document(doc)
                            .expect("Error adding document");
                    }
                    Err(e) => {
                        eprintln!("Failed to read file {}: {}", path.display(), e);
                    }
                }
            }
        }

        index_writer.commit().expect("Error committing the index");

        let reader = index.reader().expect("Error creating the index reader");

        SearchEngine {
            reader,
            index,
            lru_cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(100).expect("Error creating LRU cache"),
            )),
            default_results: default_docs,
        }
    }

    pub fn exec_query(&self, params: &Params) -> SearchResult {
        let limit = match &params.limit {
            Some(l) => l.value(),
            None => DEFAULT_SEARCH_LIMIT.value(),
        };

        let is_empty_query = params.query.is_none()
            || params.query.as_ref().unwrap().is_empty()
            || params.query.as_ref().unwrap() == "*";

        if is_empty_query {
            let cache_key = format!("{}-{}-DefaultPosts", params.query.as_ref().unwrap(), limit);
            {
                let mut cache = self.lru_cache.lock().unwrap();
                if let Some(val) = cache.get(&cache_key) {
                    return val.clone();
                }
            }

            return self.query_default_docs();
        }
        let query = params.query.as_ref().unwrap();
        let cache_key = format!("{}-{}", query, limit);

        // Try to get from cache first
        {
            let mut cache = self.lru_cache.lock().unwrap();
            if let Some(val) = cache.get(&cache_key) {
                return val.clone();
            }
        } // Lock is released here

        // Not in cache, execute query
        self.exec_query_internal(query, limit)
    }

    fn exec_query_internal(&self, query: &str, result_limit: usize) -> SearchResult {
        let searcher = self.reader.searcher();
        let schema = self.index.schema();

        let fields: Vec<Field> = schema.fields().map(|(field, _field_entry)| field).collect();
        let title_field: Field = *fields
            .iter()
            .find(|&&f| schema.get_field_name(f) == "title")
            .expect("Error while getting 'title' Field");
        let body_field: Field = *fields
            .iter()
            .find(|&&f| schema.get_field_name(f) == "body")
            .expect("Error while getting 'body' Field");

        let regex_pattern =
            Arc::new(Regex::new(format!(".*{}.*", query).as_str()).expect("Invalid regex pattern"));

        let title_regex_query = RegexQuery::from_regex(regex_pattern.clone(), title_field);
        let body_regex_query = RegexQuery::from_regex(regex_pattern.clone(), body_field);

        // Combine them with BooleanQuery
        let boolean_query = BooleanQuery::new(vec![
            (Occur::Should, Box::new(title_regex_query)),
            (Occur::Should, Box::new(body_regex_query)),
        ]);

        let top_docs = searcher
            .search(
                &boolean_query,
                &tantivy::collector::TopDocs::with_limit(result_limit),
            )
            .expect("Error while searching top documents");

        let docs: Vec<MatchingFile> = top_docs
            .iter()
            .map(|(_score, doc_address)| {
                let retrieved_doc: TantivyDocument = searcher
                    .doc(*doc_address)
                    .expect("Error while retrieving the document");

                // Extract the file path
                let file_path = retrieved_doc
                    .get_first(title_field)
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Extract the filename
                let file_name = std::path::Path::new(&file_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                MatchingFile {
                    name: file_name,
                    path: file_path,
                }
            })
            .collect();

        let search_result = SearchResult {
            matching_files: docs,
        };

        let cache_key = format!("{}-{}", query, result_limit);
        self.lru_cache
            .lock()
            .unwrap()
            .put(cache_key, search_result.clone());

        search_result
    }

    fn query_default_docs(&self) -> SearchResult {
        let files: Vec<MatchingFile> = self
            .default_results
            .iter()
            .map(|file_path| (extract_file_name(file_path.as_str()), file_path))
            .filter(|(file_name, _path)| file_name.is_some())
            .map(|(file_name, file_path)| MatchingFile {
                name: file_name.unwrap(),
                path: file_path.to_string(),
            })
            .collect();

        SearchResult {
            matching_files: files,
        }
    }
}

fn extract_file_name(path: &str) -> Option<String> {
    let file_path = std::path::Path::new(path).file_name();
    match file_path {
        Some(file_name) => file_name.to_str().map(|s| s.to_string()),
        None => {
            eprintln!("File not found in path: {}", path);
            None
        }
    }
}
