use actix_files as fs;
use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use env_logger::Env;

mod i18n;
mod routes;

pub struct AppState {
    http_caller: reqwest::Client,
    supabase_url: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    dotenv::dotenv().expect("Missing env variables");
    let client = reqwest::Client::builder()
        .build()
        .expect("Error creating HTTP client");

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(AppState {
                http_caller: client,
                supabase_url: std::env::var("SUPABASE_URL").expect("Missing env SUPABASE_URL"),
            }))
            .service(fs::Files::new("/_static", "static").show_files_listing())
            .service(routes::landing_page)
            .service(routes::articles_page)
            .service(routes::article_page)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
