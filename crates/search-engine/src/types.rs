use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

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
            // Query strings always deserialize as text, e.g. `?limit=50`
            Limit::String(val) => val
                .trim()
                .parse()
                .unwrap_or_else(|_| DEFAULT_SEARCH_LIMIT.value()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl GraphData {
    pub fn empty() -> GraphData {
        GraphData {
            nodes: vec![],
            edges: vec![],
        }
    }
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

#[derive(SurrealValue, Deserialize, Debug)]
pub struct EdgeFilePath {
    pub source: String,
    pub target: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_limit_is_used_as_is() {
        assert_eq!(Limit::Number(5).value(), 5);
    }

    #[test]
    fn numeric_text_limit_is_parsed() {
        assert_eq!(Limit::String("50".to_string()).value(), 50);
        assert_eq!(Limit::String(" 7 ".to_string()).value(), 7);
    }

    #[test]
    fn non_numeric_limit_falls_back_to_the_default() {
        assert_eq!(
            Limit::String("lots".to_string()).value(),
            DEFAULT_SEARCH_LIMIT.value()
        );
    }

    #[test]
    fn empty_graph_has_no_nodes_or_edges() {
        let graph = GraphData::empty();
        assert!(graph.nodes.is_empty() && graph.edges.is_empty());
    }
}
