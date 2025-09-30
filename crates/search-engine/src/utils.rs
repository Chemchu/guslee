use std::fs;

use gray_matter::Matter;
use serde::{Deserialize, Serialize};

pub struct TitleField;
pub struct TagsField;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MdMetadata {
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub file_name: String,
    pub file_path: String,
    pub metadata: MdMetadata,
    pub content: String,
}

pub fn extract_full_metadata(post_path: &str) -> Option<MdMetadata> {
    use gray_matter::engine::YAML;
    let matter = Matter::<YAML>::new();
    let content = fs::read_to_string(post_path).ok()?;

    match matter.parse::<MdMetadata>(&content) {
        Ok(result) => result.data,
        Err(e) => {
            eprintln!("Failed to parse frontmatter: {:?}", e);
            eprintln!("Content: {}", content);
            None
        }
    }
}
