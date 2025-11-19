use serde::{Deserialize, Serialize};

use crate::utils::Post;

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
    pub matching_posts: Vec<Post>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphNode {
    pub id: usize,
    pub label: String,
    pub file_path: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphEdge {
    pub source: usize,
    pub target: usize,
}
