use std::fs;

use gray_matter::Matter;
use serde::{Deserialize, Serialize};

pub struct TitleField;
pub struct TagsField;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostMetadata {
    pub title: String,
    pub topic: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub date: String,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MdMetadata {
    pub title: String,
    pub topic: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub date: String,
    #[serde(default)]
    pub is_draft: bool,
    pub post_source_url: String,
    pub reading_time: u8,
}

pub fn extract_full_metadata(repo_source: &str, post_path: &str) -> Option<MdMetadata> {
    use gray_matter::engine::YAML;
    let matter = Matter::<YAML>::new();
    let content = fs::read_to_string(post_path).ok()?;

    let post_metadata = match matter.parse::<PostMetadata>(&content) {
        Ok(result) => result.data,
        Err(e) => {
            eprintln!("Failed to parse frontmatter: {:?}", e);
            eprintln!("Content: {}", content);
            None
        }
    }?;

    Some(MdMetadata {
        title: post_metadata.title,
        topic: post_metadata.topic,
        description: post_metadata.description,
        tags: post_metadata.tags,
        date: post_metadata.date,
        is_draft: post_metadata.is_draft,
        post_source_url: format!(
            "{}{}",
            repo_source,
            post_path.split_once(".").unwrap_or(("", post_path)).1
        ),
        reading_time: calc_reading_time(post_path),
    })
}

pub fn calc_reading_time(post_path: &str) -> u8 {
    let avg_reading_speed = 200.0; // word per minute
    (fs::read_to_string(post_path)
        .unwrap_or("".to_string())
        .split_whitespace()
        .count() as f32
        / avg_reading_speed)
        .ceil() as u8
}
