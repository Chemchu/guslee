use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use maud::html;
use serde::Deserialize;

use crate::controllers::{AppState, render_page};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(graph_network)
        .service(garden_view_dispatcher)
        .service(garden_view);
}

#[derive(Deserialize)]
struct GraphQuery {
    depth: Option<usize>,
}

const MAX_GRAPH_DEPTH: usize = 3;

#[get("/graph/{current_url_pathname:.*}")]
async fn graph_network(
    app_state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<GraphQuery>,
) -> Html {
    let depth = query.depth.unwrap_or(1).clamp(1, MAX_GRAPH_DEPTH);
    let graph_data = {
        let file_path = format!("{}.md", path.as_str());
        app_state
            .post_search_engine
            .get_graph_from_related_posts(&file_path, depth)
            .await
    };

    let nodes_json = serde_json::to_string(&graph_data.nodes).unwrap();
    let edges_json = serde_json::to_string(&graph_data.edges).unwrap();
    let folders_json = top_folders_json(&app_state).await;

    let graph = html! {
        div class="relative w-full h-full" {
            div #graph-container
                style="width: 100%; height: 100%;"
                data-nodes=(nodes_json)
                data-edges=(edges_json)
                data-folders=(folders_json)
                data-post-path=(path.as_str())
                data-depth=(depth) {}
            div
            role="group"
            aria-label="Link depth"
            title="How many links away to show"
            class="absolute top-1 right-1 flex gap-1 text-xs"
            {
                @for option in 1..=MAX_GRAPH_DEPTH {
                    button
                    type="button"
                    data-graph-depth=(option)
                    aria-pressed=(option == depth)
                    class="w-6 py-0.5 border border-shade-color bg-background-color/80 text-zinc-500 hover:text-bright-color aria-pressed:text-primary-color aria-pressed:border-primary-color transition-colors"
                    { (option) }
                }
            }
        }
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
        style="width: 100%; height: 100%;"
        hx-get="/garden-view"
        hx-target="#garden-view-section"
        hx-trigger="load"
        hx-swap="innerHTML"
        {}
    }
    .into_string();

    render_page(&app_state, &req, &h)
}

#[get("/garden-view")]
async fn garden_view(app_state: web::Data<AppState>) -> Html {
    let graph_data = app_state.post_search_engine.get_overall_graph_data().await;
    let nodes_json = serde_json::to_string(&graph_data.nodes).unwrap();
    let edges_json = serde_json::to_string(&graph_data.edges).unwrap();

    let folders_json = top_folders_json(&app_state).await;

    let graph = html! {
        div #garden-view-content
        style="width: 100%; height: 100%;"
        data-nodes=(nodes_json)
        data-edges=(edges_json)
        data-folders=(folders_json)
        data-root-label="garden entry" {}
    };

    Html::new(graph)
}

async fn top_folders_json(app_state: &AppState) -> String {
    let mut folders: Vec<String> = app_state
        .post_search_engine
        .published_posts()
        .await
        .into_iter()
        .filter_map(|post| Some(post.file_path.split_once('/')?.0.to_string()))
        .collect();
    folders.sort();
    folders.dedup();
    serde_json::to_string(&folders).unwrap()
}
