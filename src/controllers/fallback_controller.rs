use actix_web::{HttpRequest, Responder, get, web};

use crate::controllers::{AppState, render_not_found};

#[get("/{url:.*}")]
pub async fn fallback_route(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    render_not_found(&app_state, &req)
}
