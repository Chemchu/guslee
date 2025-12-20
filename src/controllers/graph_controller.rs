use actix_web::{
    get,
    web::{self, Html},
};
use maud::html;

use crate::controllers::AppState;

#[get("/graph/{current_url_pathname:.*}")]
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

#[get("/garden-view")]
pub async fn garden_view(app_state: web::Data<AppState>) -> Html {
    let graph_data = app_state.search_engine.get_overall_graph_data().await;
    let nodes_json = serde_json::to_string(&graph_data.nodes).unwrap();
    let edges_json = serde_json::to_string(&graph_data.edges).unwrap();

    let graph = html! {
        div #garden-view-section
            style="width: 100%; min-height: 80vh;"
            data-nodes=(nodes_json)
            data-edges=(edges_json) {}
    };

    Html::new(graph)
}
