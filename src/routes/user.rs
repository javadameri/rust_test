use actix_web::{web};
use crate::controllers::user_controller::*;


pub fn config_routes(cfg: &mut web::ServiceConfig) {
    // مسیرهای ثبت‌نام و لاگین
    cfg.service(web::resource("/register").route(web::post().to(register)));
    cfg.service(web::resource("/login").route(web::post().to(login)));

    // مسیرهای مدیریت نقش‌ها و دسترسی‌ها
    cfg.service(web::resource("/roles").route(web::post().to(add_role)));
    cfg.service(web::resource("/permissions").route(web::post().to(add_permission)));

    // مسیرهای مدیریت روابط دسترسی‌ها به نقش‌ها
    cfg.service(web::resource("/role_permissions").route(web::post().to(add_role_permission)));

    // مسیرهای مدیریت روابط نقش‌ها به کاربران
    cfg.service(web::resource("/assign_role_to_user").route(web::post().to(assign_role_to_user)));

    // مسیرهای دریافت اطلاعات نقش‌ها و دسترسی‌ها
    cfg.service(web::resource("/roles/{user_id}").route(web::get().to(get_roles_for_user)));
    cfg.service(web::resource("/permissions/{role_id}").route(web::get().to(get_permissions_for_role)));
}
