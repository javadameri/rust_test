use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use crate::schema::items::dsl::*;
use crate::config::DbPool;
use crate::models::item::{Item, NewItem};

pub async fn get_items(pool: web::Data<DbPool>) -> impl Responder {
    let conn = pool.get().expect("Couldn't get db connection from pool");
    let result = web::block(move || {
        let mut conn = conn;
        items.load::<Item>(&mut conn)
    })
    .await;

    match result {
        Ok(Ok(item_list)) => HttpResponse::Ok().json(item_list),
        Ok(Err(query_err)) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}

pub async fn create_item(
    pool: web::Data<DbPool>,
    new_item: web::Json<NewItem>,
) -> impl Responder {
    let conn = pool.get().expect("Couldn't get db connection from pool");
    let new_item = new_item.into_inner();
    let result = web::block(move || {
        let mut conn = conn;
        diesel::insert_into(items)
            .values(new_item)
            .get_result::<Item>(&mut conn)
    })
    .await;

    match result {
        Ok(Ok(inserted_item)) => HttpResponse::Ok().json(inserted_item),
        Ok(Err(query_err)) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}

pub async fn update_item(
    pool: web::Data<DbPool>,
    item_id: web::Path<i32>,
    updated_item: web::Json<NewItem>,
) -> impl Responder {
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
        Ok(Ok(item)) => HttpResponse::Ok().json(item),
        Ok(Err(query_err)) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}

pub async fn delete_item(
    pool: web::Data<DbPool>,
    item_id: web::Path<i32>,
) -> impl Responder {
    let conn = pool.get().expect("Couldn't get db connection from pool");
    let target_id = item_id.into_inner();
    let result = web::block(move || {
        let mut conn = conn;
        diesel::delete(items.find(target_id)).execute(&mut conn)
    })
    .await;

    match result {
        Ok(Ok(_)) => HttpResponse::Ok().body("Item deleted"),
        Ok(Err(query_err)) => HttpResponse::InternalServerError().body(format!("Query error: {}", query_err)),
        Err(blocking_err) => HttpResponse::InternalServerError().body(format!("Blocking error: {}", blocking_err)),
    }
}
