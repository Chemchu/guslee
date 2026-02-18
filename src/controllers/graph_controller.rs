use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use maud::html;

use crate::controllers::{AppState, wrap_content_into_full_page};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(graph_network)
        .service(garden_view_dispatcher)
        .service(garden_view);
}

#[get("/graph/{current_url_pathname:.*}")]
async fn graph_network(app_state: web::Data<AppState>, path: web::Path<String>) -> Html {
    let graph_data = {
        let file_path = format!("{}.md", path.as_str());
        app_state
            .post_search_engine
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

#[get("/garden-view-dispatcher")]
async fn garden_view_dispatcher(
    app_state: web::Data<AppState>,
    req: HttpRequest,
) -> impl actix_web::Responder {
    let h = html! {
        div
        #garden-view-section
        style="width: 100%; min-height: 90vh;"
        hx-get="/garden-view"
        hx-target="#garden-view-section"
        hx-trigger="load"
        hx-swap="innerHTML"
        {}
    }
    .into_string();

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(h)
    } else {
        Html::new(wrap_content_into_full_page(&app_state.app_name, &h))
    }
}

#[get("/garden-view")]
async fn garden_view(app_state: web::Data<AppState>) -> Html {
    let graph_data = app_state.post_search_engine.get_overall_graph_data().await;
    let nodes_json = serde_json::to_string(&graph_data.nodes).unwrap();
    let edges_json = serde_json::to_string(&graph_data.edges).unwrap();

    let graph = html! {
        div #garden-view-content
        style="width: 100%; min-height: 80vh;"
        data-nodes=(nodes_json)
        data-edges=(edges_json) {}
    };

    Html::new(graph)
}
