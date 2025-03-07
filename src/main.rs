#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

mod schema {
    table! {
        items (id) {
            id -> Int4,
            name -> Varchar,
            created_at -> Timestamp,
        }
    }
}

mod models;
use models::{Item, NewItem};

async fn get_items(pool: web::Data<DbPool>) -> impl Responder {
    use schema::items::dsl::*;
    let conn = pool.get().expect("Couldn't get db connection from pool");
    let result = web::block(move || {
        let mut conn = conn;
        items.load::<Item>(&mut conn)
    })
    .await;
    match result {
        Ok(inner) => match inner {
            Ok(item_list) => HttpResponse::Ok().json(item_list),
            Err(query_err) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        },
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}

async fn create_item(
    pool: web::Data<DbPool>,
    new_item: web::Json<NewItem>,
) -> impl Responder {
    use schema::items;
    let conn = pool.get().expect("Couldn't get db connection from pool");
    let new_item = new_item.into_inner();
    let result = web::block(move || {
        let mut conn = conn;
        diesel::insert_into(items::table)
            .values(new_item)
            .get_result::<Item>(&mut conn)
    })
    .await;
    match result {
        Ok(inner) => match inner {
            Ok(inserted_item) => HttpResponse::Ok().json(inserted_item),
            Err(query_err) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        },
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}

async fn update_item(
    pool: web::Data<DbPool>,
    item_id: web::Path<i32>,
    updated_item: web::Json<NewItem>,
) -> impl Responder {
    use schema::items::dsl::*;
    let conn = pool.get().expect("Couldn't get db connection from pool");
    let target_id = item_id.into_inner();
    let new_data = updated_item.into_inner();
    let result = web::block(move || {
        let mut conn = conn;
        diesel::update(items.find(target_id))
            .set(name.eq(new_data.name))
            .get_result::<Item>(&mut conn)
    })
    .await;
    match result {
        Ok(inner) => match inner {
            Ok(item) => HttpResponse::Ok().json(item),
            Err(query_err) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        },
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}

async fn delete_item(
    pool: web::Data<DbPool>,
    item_id: web::Path<i32>,
) -> impl Responder {
    use schema::items::dsl::*;
    let conn = pool.get().expect("Couldn't get db connection from pool");
    let target_id = item_id.into_inner();
    let result = web::block(move || {
        let mut conn = conn;
        diesel::delete(items.find(target_id)).execute(&mut conn)
    })
    .await;
    match result {
        Ok(inner) => match inner {
            Ok(_) => HttpResponse::Ok().body("Item deleted"),
            Err(query_err) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        },
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let host = env::var("HOST").unwrap_or("127.0.0.1:8080".to_string());
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/items", web::get().to(get_items))
            .route("/items", web::post().to(create_item))
            .route("/items/{id}", web::put().to(update_item))
            .route("/items/{id}", web::delete().to(delete_item))
    })
    .bind(host)? // سرور روی 127.0.0.1:8080 اجرا می‌شود
    .run()
    .await
}
