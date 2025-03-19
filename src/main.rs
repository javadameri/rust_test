extern crate diesel;
extern crate dotenv;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use crate::config::establish_connection;
use crate::routes::items::config_routes;
use crate::routes::user::config_routes as user_routes;

mod config;
mod models;
mod controllers;
mod routes;
mod schema;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let host = env::var("HOST").unwrap_or("127.0.0.1:8080".to_string());
    let pool = establish_connection();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(config_routes)
            .configure(user_routes)
    })
    .bind(host)?
    .run()
    .await
}
