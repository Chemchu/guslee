use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum Limit {
    Number(usize),
    String(String),
}

#[derive(Deserialize, Clone)]
pub struct Params {
    pub query: Option<String>,
    pub limit: Option<Limit>,
}

pub const DEFAULT_SEARCH_LIMIT: Limit = Limit::Number(100);

impl Limit {
    pub fn value(&self) -> usize {
        match self {
            Limit::Number(val) => *val,
            Limit::String(_val) => DEFAULT_SEARCH_LIMIT.value(),
        }
    }
}

#[derive(Clone)]
pub struct SearchResult {
    pub matching_files: Vec<MatchingFile>,
}

#[derive(Clone)]
pub struct MatchingFile {
    title: String,
    path: String,
}

impl MatchingFile {
    pub fn new(name: String, path: String) -> Self {
        MatchingFile { title: name, path }
    }
    pub fn file_name(&self) -> &str {
        &self.title
    }

    pub fn file_path(&self) -> &str {
        &self.path
    }
}
