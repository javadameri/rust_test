use actix_web::web;
use crate::controllers::items_controller::*;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/items")
            .route("", web::get().to(get_items))
            .route("", web::post().to(create_item))
            .route("/{id}", web::put().to(update_item))
            .route("/{id}", web::delete().to(delete_item)),
    );
}
