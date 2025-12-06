use actix_web::{
    get,
    web::{self, Html},
};
use maud::html;

use crate::controllers::AppState;

#[get("/graph/{current_url_pathname}")]
pub async fn graph_network(app_state: web::Data<AppState>, path: web::Path<String>) -> Html {
    let graph_data = {
        let file_path = format!("{}.md", path.as_str());
        app_state
            .search_engine
            .get_graph_from_related_posts(&file_path)
            .await
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
