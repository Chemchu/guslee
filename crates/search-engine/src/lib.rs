use std::fs;
use tantivy::{
    Document, Index, IndexReader, IndexWriter, TantivyDocument,
    schema::{Field, STORED, Schema, TEXT},
};
use tempdir::TempDir;
use walkdir::WalkDir;

pub struct SearchEngine {
    reader: IndexReader,
}

impl SearchEngine {
    pub fn new(documents_path: &str) -> SearchEngine {
        let index_path =
            TempDir::new("index").expect("Temporary directory 'index' creation failed");
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT);
        let schema = schema_builder.build();
        let index =
            Index::create_in_dir(index_path.path(), schema.clone()).expect("Index creation failed");
        let mut index_writer: IndexWriter = index
            .writer(50_000_000)
            .expect("Failed to allocate 50MB to index");
        let title: Field = schema.get_field("title").unwrap();
        let body: Field = schema.get_field("body").unwrap();

        // Walk through the garden directory recursively
        for entry in WalkDir::new(documents_path) {
            let entry = entry.expect("Error while accessing the WalkDir entry");
            let path = entry.path();

            if path.is_file()
                && let Ok(relative_path) = path.strip_prefix(documents_path)
            {
                let file_path = relative_path.to_string_lossy().to_string();
                println!("Relative path: {}", file_path);

                // Read file content
                match fs::read_to_string(path) {
                    Ok(content) => {
                        let mut doc = TantivyDocument::default();
                        doc.add_text(title, &file_path);
                        doc.add_text(body, &content);

                        // Add document to index
                        index_writer
                            .add_document(doc)
                            .expect("Error while adding a document to the index writer");
                        println!("Added document: {}", file_path);
                    }
                    Err(e) => {
                        eprintln!("Failed to read file {}: {}", path.display(), e);
                    }
                }
            }
        }

        index_writer.commit().expect("Error commiting the index");
        println!("Successfully indexed all documents from garden folder");

        let reader = index
            .reader_builder()
            .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .expect("Error creating the index reader");

        SearchEngine { reader }
    }
    pub fn exec_query(&self, query: &str, result_limit: usize) {
        let searcher = self.reader.searcher();
        let index = searcher.index();
        let schema = index.schema();
        let fields = schema.fields().map(|(field, _field_entry)| field).collect();

        let query_parser = tantivy::query::QueryParser::for_index(index, fields);
        let query = query_parser
            .parse_query(query)
            .unwrap_or_else(|_| panic!("Error while parsing the query: {}", query));

        let top_docs = searcher
            .search(
                &query,
                &tantivy::collector::TopDocs::with_limit(result_limit),
            )
            .expect("Error while searching top documents");

        for (_score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher
                .doc(doc_address)
                .expect("Error while retrieving the document");
            println!("{}", retrieved_doc.to_json(&schema));
        }
    }
}
