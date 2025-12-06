use actix_web::{
    get,
    web::{self, Html},
};
use maud::html;

use crate::controllers::AppState;

#[get("/music/{username}")]
pub async fn get_user_profile(app_state: web::Data<AppState>, path: web::Path<String>) -> Html {
    let username = path.as_str();
    let html = html! {};

    todo!()
}
