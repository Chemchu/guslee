use std::fs;

use gray_matter::Matter;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct MdMetadata {
    title: String,
    tags: Vec<String>,
}

pub struct TitleField;
pub struct TagsField;

impl TitleField {
    pub const ID: u8 = 1;
}
impl TagsField {
    pub const ID: u8 = 2;
}

macro_rules! define_field_extractor {
    ($field_type:ty, $return_type:ty, $field_access:expr) => {
        impl $field_type {
            pub fn extract(path: &str) -> Option<$return_type> {
                let metadata = extract_metadata(path)?;
                Some($field_access(&metadata))
            }
        }
    };
}

define_field_extractor!(TitleField, String, |m: &MdMetadata| m.title.clone());
define_field_extractor!(TagsField, Vec<String>, |m: &MdMetadata| m.tags.clone());

fn extract_metadata(markdown_path: &str) -> Option<MdMetadata> {
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
