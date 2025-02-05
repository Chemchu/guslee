use std::sync::Arc;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use http_service::HttpService;

mod http_service;
mod i18n;
mod md_service;
mod routes;

struct AppState {
    http_service: Arc<HttpService>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    dotenv().ok();

    let supabase_url = std::env::var("SUPABASE_URL").expect("Missing env SUPABASE_URL");
    let api_key = std::env::var("SUPABASE_KEY").expect("Missing env SUPABASE_KEY");
    let port = std::env::var("PORT").expect("Missing env PORT");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(AppState {
                http_service: Arc::new(HttpService::new(
                    api_key.to_owned(),
                    supabase_url.to_owned(),
                )),
            }))
            .service(actix_files::Files::new("/_static", "static").show_files_listing())
            .service(routes::landing_page)
            .service(routes::articles_page)
            .service(routes::article_page)
    })
    .bind(("0.0.0.0", port.parse::<u16>().unwrap()))?
    .run()
    .await
}
