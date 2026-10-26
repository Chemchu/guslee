use actix_web::{
    App, Error,
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    middleware::{Compress, Logger},
    web,
};

use crate::controllers::AppState;

pub mod controllers;
pub mod helpers;

pub fn create_app(
    app_state: web::Data<AppState>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse<impl MessageBody>,
        Error = Error,
        InitError = (),
    >,
> {
    App::new()
        .wrap(Compress::default())
        .wrap(Logger::new("%s %r - %Dms"))
        .service(actix_files::Files::new("/_static", "./static"))
        .app_data(app_state)
        .configure(controllers::posts_controller::configure_services)
        .configure(controllers::news_controller::configure_services)
        .configure(controllers::metadata_controller::configure_services)
        .configure(controllers::steam_controller::configure_services)
        .configure(controllers::chess_controller::configure_services)
        .configure(controllers::graph_controller::configure_services)
        .configure(controllers::music_controller::configure_services)
        .configure(controllers::routines_controller::configure_services)
        .configure(controllers::tags_controller::configure_services)
        .configure(controllers::feed_controller::configure_services)
        .service(controllers::fallback_controller::fallback_route) // This service should be last one in the list because it matches any string
}
