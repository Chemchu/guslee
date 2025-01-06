use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use dotenv::dotenv;
use env_logger::Env;
use reqwest::Client;
use std::sync::Arc;

mod i18n;
mod routes;

struct AppState {
    http_caller: Arc<Client>,
    supabase_url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    dotenv().expect("Missing env variables");
    let client = Arc::new(
        reqwest::Client::builder()
            .build()
            .expect("Error creating HTTP client"),
    );

    let supabase_url = std::env::var("SUPABASE_URL").expect("Missing env SUPABASE_URL");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(AppState {
                http_caller: client.clone(),
                supabase_url: supabase_url.clone(),
            }))
            .service(actix_files::Files::new("/_static", "static").show_files_listing())
            .service(routes::landing_page)
            .service(routes::articles_page)
            .service(routes::article_page)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
