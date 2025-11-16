use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use maud::html;

use crate::routes::AppState;

#[get("/graph")]
pub async fn graph_network(app_state: web::Data<AppState>, req: HttpRequest) -> Html {
    let current_url = req.headers().get("HX-Current-URL");
    let graph_data = if let Some(current_url) = current_url {
        let result: Vec<&str> = current_url.to_str().unwrap().splitn(4, '/').collect();
        let file_name = result
            .get(3)
            .filter(|s| !s.is_empty())
            .unwrap_or(&"welcome");
        let file_path = format!("{}.md", file_name);
        app_state
            .search_engine
            .get_graph_from_related_posts(&file_path)
            .await
    } else {
        search_engine::types::GraphData {
            nodes: vec![],
            edges: vec![],
        }
    };

    let nodes_json = serde_json::to_string(&graph_data.nodes).unwrap();
    let edges_json = serde_json::to_string(&graph_data.edges).unwrap();

    let graph = html! {
        div #graph-container
            style="width: 100%; height: 100%;"
            data-nodes=(nodes_json)
            data-edges=(edges_json) {}
    };

    Html::new(graph)
}
