use std::collections::HashMap;
use std::fs;

use gray_matter::Matter;
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

pub const DEFAULT_LANG: &str = "en";

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
    pub lang: String,
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
    #[serde(default)]
    pub last_updated: Option<String>,
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
        last_updated: None,
    })
}

fn source_url(repo_source: &str, post_path: &str) -> String {
    format!(
        "{}/{}",
        repo_source.trim_end_matches('/'),
        post_path.trim_start_matches("./")
    )
}

pub fn split_translation(file_path: &str) -> (String, &str) {
    let translation = file_path
        .strip_suffix(".md")
        .and_then(|stem| stem.rsplit_once('.'))
        .filter(|(name, lang)| {
            !name.is_empty()
                && !name.ends_with('/')
                && lang.len() == 2
                && lang.bytes().all(|b| b.is_ascii_lowercase())
        });

    match translation {
        Some((name, lang)) => (format!("{name}.md"), lang),
        None => (file_path.to_string(), DEFAULT_LANG),
    }
}

fn lang_rank<'a>(post: &'a Post, lang: &str) -> (bool, bool, &'a str) {
    (post.lang != lang, post.lang != DEFAULT_LANG, &post.lang)
}

pub fn localize(posts: Vec<Post>, lang: &str) -> Vec<Post> {
    let mut order: Vec<String> = Vec::new();
    let mut best: HashMap<String, Post> = HashMap::new();

    for post in posts {
        match best.get(&post.file_path) {
            None => {
                order.push(post.file_path.clone());
                best.insert(post.file_path.clone(), post);
            }
            Some(current) if lang_rank(&post, lang) < lang_rank(current, lang) => {
                best.insert(post.file_path.clone(), post);
            }
            Some(_) => {}
        }
    }

    order
        .into_iter()
        .filter_map(|file_path| best.remove(&file_path))
        .collect()
}

pub fn strip_frontmatter(content: &str) -> &str {
    let Some(rest) = content.strip_prefix("---\n") else {
        return content;
    };
    match rest.find("\n---") {
        Some(end) => {
            let after = &rest[end + "\n---".len()..];
            after.strip_prefix('\n').unwrap_or(after)
        }
        None => content,
    }
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
        assert_eq!(
            source_url(&format!("{repo}/"), "garden/x.md"),
            "https://github.com/u/r/blob/main/garden/x.md"
        );
    }

    #[test]
    fn frontmatter_is_stripped_from_the_body() {
        assert_eq!(strip_frontmatter("---\ntitle: x\n---\n# Body"), "# Body");
        assert_eq!(strip_frontmatter("# No frontmatter"), "# No frontmatter");
        assert_eq!(strip_frontmatter("---\nunclosed"), "---\nunclosed");
    }

    fn post(file_path: &str, lang: &str) -> Post {
        Post {
            file_name: file_path.rsplit('/').next().unwrap().to_string(),
            file_path: file_path.to_string(),
            lang: lang.to_string(),
            metadata: MdMetadata {
                title: format!("{file_path} ({lang})"),
                topic: None,
                description: String::new(),
                tags: vec![],
                date: String::new(),
                is_draft: false,
                post_source_url: String::new(),
                reading_time: 0,
                last_updated: None,
            },
            content: String::new(),
        }
    }

    fn picked(posts: &[Post]) -> Vec<(&str, &str)> {
        posts
            .iter()
            .map(|p| (p.file_path.as_str(), p.lang.as_str()))
            .collect()
    }

    #[test]
    fn translation_suffixes_are_split_from_the_path() {
        assert_eq!(
            split_translation("gaming/peak/mesa_update.es.md"),
            ("gaming/peak/mesa_update.md".to_string(), "es")
        );
        assert_eq!(
            split_translation("gaming/peak/mesa_update.md"),
            ("gaming/peak/mesa_update.md".to_string(), DEFAULT_LANG)
        );
        assert_eq!(
            split_translation("welcome.fr.md"),
            ("welcome.md".to_string(), "fr")
        );
    }

    #[test]
    fn dotted_names_that_are_not_languages_stay_untouched() {
        for path in [
            "v1.2.md",
            "notes.ES.md",
            "notes.eng.md",
            "dir.es/post.md",
            "notes/.es.md",
        ] {
            assert_eq!(split_translation(path), (path.to_string(), DEFAULT_LANG));
        }
    }

    #[test]
    fn localize_prefers_the_requested_language_then_the_default() {
        let posts = vec![
            post("a.md", "en"),
            post("a.md", "es"),
            post("b.md", "en"),
            post("c.md", "fr"),
            post("c.md", "es"),
        ];

        assert_eq!(
            picked(&localize(posts.clone(), "es")),
            vec![("a.md", "es"), ("b.md", "en"), ("c.md", "es")]
        );
        assert_eq!(
            picked(&localize(posts.clone(), "en")),
            vec![("a.md", "en"), ("b.md", "en"), ("c.md", "es")]
        );
        assert_eq!(
            picked(&localize(posts, "de")),
            vec![("a.md", "en"), ("b.md", "en"), ("c.md", "es")]
        );
    }

    #[test]
    fn localize_keeps_the_order_of_first_appearance() {
        let posts = vec![post("b.md", "es"), post("a.md", "en"), post("b.md", "en")];

        assert_eq!(
            picked(&localize(posts, "en")),
            vec![("b.md", "en"), ("a.md", "en")]
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
