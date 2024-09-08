use actix_files as fs;
use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;
use i18n::translator::Language;

mod i18n;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let translator = i18n::translator::Translator::new();
    let en = translator.get_translation("test1", &Language::English);
    let es = translator.get_translation("test1", &Language::Spanish);
    let pt = translator.get_translation("test1", &Language::Portuguese);

    println!("All languages {:?}, {:?} and {:?}", en, es, pt);

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(fs::Files::new("/_static", "static").show_files_listing())
            .service(routes::landing_page)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
