use std::fs;

use actix_web::{App, HttpServer, web};
use tantivy::{
    Document, Index, IndexReader, IndexWriter, TantivyDocument,
    schema::{STORED, Schema, TEXT},
};
use tempdir::TempDir;
use walkdir::WalkDir;

use crate::routes::AppState;

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(actix_files::Files::new("/_static", "./static").show_files_listing())
            .app_data(web::Data::new(AppState {
                app_name: String::from("Gustavo's digital garden"),
                posts_reader: init_posts(),
            }))
            .service(routes::landing)
            .service(routes::post)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}

/// TODO: finish and move to a different module
fn init_posts() -> IndexReader {
    let index_path =
        TempDir::new("posts_index").expect("Temporary directory 'post_index' creation failed");
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    let schema = schema_builder.build();
    let index =
        Index::create_in_dir(index_path.path(), schema.clone()).expect("Index creation failed");
    let mut index_writer: IndexWriter = index
        .writer(50_000_000)
        .expect("Failed to allocate 50MB to index");
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    // Walk through the garden directory recursively
    for entry in WalkDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/garden")) {
        let entry = entry.expect("Error while accessing the WalkDir entry");
        let path = entry.path();

        // Only process files (not directories)
        if path.is_file() {
            // Get the filename (without path) as title
            let file_title = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("untitled");

            // Read file content
            match fs::read_to_string(path) {
                Ok(content) => {
                    // Create a new document
                    let mut doc = TantivyDocument::default();
                    doc.add_text(title, file_title);
                    doc.add_text(body, &content);

                    // Add document to index
                    index_writer
                        .add_document(doc)
                        .expect("Error while adding a document to the index writer");
                    println!("Added document: {}", file_title);
                }
                Err(e) => {
                    eprintln!("Failed to read file {}: {}", path.display(), e);
                    // Continue processing other files
                }
            }
        }
    }

    // Commit all documents to the index
    index_writer.commit().expect("Error commiting the index");
    println!("Successfully indexed all documents from garden folder");

    let reader = index
        .reader_builder()
        .reload_policy(tantivy::ReloadPolicy::OnCommitWithDelay)
        .try_into()
        .expect("Error creating the index reader");

    let searcher = reader.searcher();
    let query_parser = tantivy::query::QueryParser::for_index(&index, vec![title, body]);
    let query = query_parser
        .parse_query("styling")
        .expect("Error while parsing the query");

    let top_docs = searcher
        .search(&query, &tantivy::collector::TopDocs::with_limit(10))
        .expect("Error while searching top documents");
    for (_score, doc_address) in top_docs {
        let retrieved_doc: TantivyDocument = searcher
            .doc(doc_address)
            .expect("Error while retrieving the document");
        println!("{}", retrieved_doc.to_json(&schema));
    }

    reader
}
