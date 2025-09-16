use std::fs;

use gray_matter::Matter;
use serde::Deserialize;

pub struct TitleField;
pub struct TagsField;

#[derive(Deserialize, Debug, Clone)]
struct MdMetadata {
    title: String,
    tags: Vec<String>,
}

pub trait MetadataField<T> {
    fn data(path: &str) -> T;
}

impl MetadataField<Option<String>> for TitleField {
    fn data(path: &str) -> Option<String> {
        match extract_metadata_from_file_in_path(path) {
            Some(metadata) => Some(metadata.title),
            None => None,
        }
    }
}

impl MetadataField<Option<Vec<String>>> for TagsField {
    fn data(path: &str) -> Option<Vec<String>> {
        match extract_metadata_from_file_in_path(path) {
            Some(metadata) => Some(metadata.tags),
            None => None,
        }
    }
}

pub fn extract_metadata<F: MetadataField<T>, T>(path: &str) -> T {
    F::data(path)
}

fn extract_metadata_from_file_in_path(markdown_path: &str) -> Option<MdMetadata> {
    use gray_matter::engine::YAML;
    let matter = Matter::<YAML>::new();

    let content = fs::read_to_string(format!("./garden/{}", markdown_path));
    match content {
        Ok(c) => {
            let result_with_struct = matter.parse::<MdMetadata>(c.as_str()).unwrap();
            result_with_struct.data
        }
        Err(_e) => None,
    }
}
