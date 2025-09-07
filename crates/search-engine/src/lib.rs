use serde::Deserialize;
use std::fs;
use tantivy::{
    Index, IndexReader, IndexWriter, Searcher, TantivyDocument,
    schema::{Field, STORED, Schema, TEXT, Value},
};
use walkdir::WalkDir;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Limit {
    Number(usize),
    String(String),
}

#[derive(Deserialize)]
pub struct Params {
    pub limit: Option<Limit>,
}

const DEFAULT_SEARCH_LIMIT: Limit = Limit::Number(100);

impl Limit {
    pub fn value(&self) -> usize {
        match self {
            Limit::Number(n) => *n,
            Limit::String(_val) => DEFAULT_SEARCH_LIMIT.value(),
        }
    }
}

pub struct SearchResult {
    pub matching_files: Vec<MatchingFile>,
}

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
}

impl SearchEngine {
    pub fn new(documents_path: &str) -> SearchEngine {
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
                        doc.add_text(title, &file_path);
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

        SearchEngine { reader, index }
    }

    pub fn exec_query(&self, query: &str, result_limit: Option<Limit>) -> SearchResult {
        let searcher = self.reader.searcher();
        let index = &self.index;
        let schema = index.schema();
        let fields: Vec<Field> = schema.fields().map(|(field, _field_entry)| field).collect();
        let title_field: Field = *fields
            .iter()
            .find(|&&f| schema.get_field_name(f) == "title")
            .expect("Error while getting 'title' Field");

        let query_parser = tantivy::query::QueryParser::for_index(index, fields);
        let query = query_parser
            .parse_query(query)
            .unwrap_or_else(|_| panic!("Error while parsing the query: {}", query));

        let top_docs = searcher
            .search(
                &query,
                &tantivy::collector::TopDocs::with_limit(
                    result_limit.unwrap_or(DEFAULT_SEARCH_LIMIT).value(),
                ),
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

        SearchResult {
            matching_files: docs,
        }
    }

    fn exec_query_internal(
        searcher: Searcher,
        index: &Index,
        query: &str,
        result_limit: Option<Limit>,
    ) -> SearchResult {
        let schema = index.schema();
        let fields: Vec<Field> = schema.fields().map(|(field, _field_entry)| field).collect();
        let title_field: Field = *fields
            .iter()
            .find(|&&f| schema.get_field_name(f) == "title")
            .expect("Error while getting 'title' Field");

        let query_parser = tantivy::query::QueryParser::for_index(index, fields);
        let query = query_parser
            .parse_query(query)
            .unwrap_or_else(|_| panic!("Error while parsing the query: {}", query));

        let top_docs = searcher
            .search(
                &query,
                &tantivy::collector::TopDocs::with_limit(
                    result_limit.unwrap_or(DEFAULT_SEARCH_LIMIT).value(),
                ),
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

        SearchResult {
            matching_files: docs,
        }
    }
}
