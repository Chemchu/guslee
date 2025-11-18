use actix_web::{
    HttpRequest, get,
    web::{self, Html},
};
use maud::html;

use crate::routes::AppState;

#[get("/metadata/{post}")]
pub async fn render_metadata(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<String>,
) -> Html {
    let current_url = req.headers().get("HX-Current-URL");

    let html = html! {
        span {
            "Metadata"
        }
    };

    Html::new(html)
}
