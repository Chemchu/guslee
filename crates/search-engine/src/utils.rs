use std::fs;

use gray_matter::Matter;
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

pub struct TitleField;
pub struct TagsField;

#[derive(SurrealValue, Serialize, Deserialize, Debug, Clone)]
pub struct PostMetadata {
    pub title: String,
    pub topic: Option<String>,
    pub description: String,
    pub tags: Vec<String>,
    pub date: String,
    #[serde(default)]
    pub is_draft: bool,
}

#[derive(SurrealValue, Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    pub file_name: String,
    pub file_path: String,
    pub metadata: MdMetadata,
    pub content: String,
}

#[derive(SurrealValue, Serialize, Deserialize, Debug, Clone)]
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
        post_source_url: source_url(repo_source, post_path),
        reading_time: calc_reading_time(post_path),
    })
}

// Link to the post in the repository, e.g. "<repo>/garden/hello.md" for "./garden/hello.md"
fn source_url(repo_source: &str, post_path: &str) -> String {
    format!(
        "{}/{}",
        repo_source.trim_end_matches('/'),
        post_path.trim_start_matches("./")
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn write_post(dir: &Path, name: &str, content: &str) -> String {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    const FRONTMATTER: &str = "---\ntitle: \"A title\"\ndate: \"2025-01-02\"\ndescription: \"A description\"\ntopic: \"Gaming\"\ntags:\n  - one\n  - two\n---\n";

    #[test]
    fn reading_time_rounds_up_at_200_words_per_minute() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(calc_reading_time(&write_post(dir.path(), "a.md", "")), 0);
        assert_eq!(
            calc_reading_time(&write_post(dir.path(), "b.md", &"word ".repeat(200))),
            1
        );
        assert_eq!(
            calc_reading_time(&write_post(dir.path(), "c.md", &"word ".repeat(201))),
            2
        );
    }

    #[test]
    fn source_url_joins_the_repo_and_the_post_path() {
        let repo = "https://github.com/u/r/blob/main";

        assert_eq!(
            source_url(repo, "./garden/journal/x.md"),
            "https://github.com/u/r/blob/main/garden/journal/x.md"
        );
        // Used to produce "<repo>md" when the path had no leading "./"
        assert_eq!(
            source_url(&format!("{repo}/"), "garden/x.md"),
            "https://github.com/u/r/blob/main/garden/x.md"
        );
    }

    #[test]
    fn reading_time_of_a_missing_file_is_zero() {
        assert_eq!(calc_reading_time("/definitely/not/here.md"), 0);
    }

    #[test]
    fn extracts_frontmatter_metadata() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_post(dir.path(), "post.md", &format!("{FRONTMATTER}Body text"));

        let metadata = extract_full_metadata("https://repo", &path).unwrap();

        assert_eq!(metadata.title, "A title");
        assert_eq!(metadata.description, "A description");
        assert_eq!(metadata.date, "2025-01-02");
        assert_eq!(metadata.topic.as_deref(), Some("Gaming"));
        assert_eq!(metadata.tags, vec!["one", "two"]);
        assert!(!metadata.is_draft, "is_draft defaults to false");
        assert_eq!(metadata.reading_time, 1);
    }

    #[test]
    fn reads_the_draft_flag() {
        let dir = tempfile::tempdir().unwrap();
        let content = FRONTMATTER.replace("---\ntitle", "---\nis_draft: true\ntitle");
        let path = write_post(dir.path(), "draft.md", &content);

        assert!(
            extract_full_metadata("https://repo", &path)
                .unwrap()
                .is_draft
        );
    }

    #[test]
    fn missing_frontmatter_or_required_fields_yield_none() {
        let dir = tempfile::tempdir().unwrap();
        let no_frontmatter = write_post(dir.path(), "plain.md", "# Just markdown");
        let no_title = write_post(
            dir.path(),
            "no_title.md",
            &FRONTMATTER.replace("title: \"A title\"\n", ""),
        );

        assert!(extract_full_metadata("https://repo", &no_frontmatter).is_none());
        assert!(extract_full_metadata("https://repo", &no_title).is_none());
        assert!(extract_full_metadata("https://repo", "/missing.md").is_none());
    }
}
