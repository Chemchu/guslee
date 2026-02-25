use std::fs;

use gray_matter::Matter;
use serde::{Deserialize, Serialize};

pub struct TitleField;
pub struct TagsField;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MdMetadata {
    pub title: String,
    pub topic: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub date: String,
    #[serde(default)]
    pub post_source_url: String,
    #[serde(default)]
    pub is_draft: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub file_name: String,
    pub file_path: String,
    pub metadata: MdMetadata,
    pub content: String,
}

pub fn extract_full_metadata(repo_source: &str, post_path: &str) -> Option<MdMetadata> {
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
    .map(|mut md_metadata| {
        md_metadata.post_source_url = format!(
            "{}{}",
            repo_source,
            post_path
                .split_once(".")
                .unwrap_or(("", post_path))
                .1
                .to_string()
        );
        md_metadata
    })
}
