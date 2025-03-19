use actix_web::{Error, HttpMessage, dev::{ServiceRequest, ServiceResponse}};
use actix_service::{Service, Transform};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, Validation, DecodingKey};
use dotenv::dotenv;
use std::env;
use std::task::{Context, Poll};
use std::future::{ready, Ready};
use crate::models::user::Claims;
use diesel::{prelude::*};
use diesel::r2d2::{ConnectionManager, Pool};
use actix_web::web;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub struct RbacMiddleware {
    required_permission: String, // مجوز مورد نیاز
}

impl RbacMiddleware {
    pub fn new(permission: &str) -> Self {
        RbacMiddleware {
            required_permission: permission.to_string(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RbacMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RbacMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RbacMiddlewareService {
            service,
            required_permission: self.required_permission.clone(),
        }))
    }
}

pub struct RbacMiddlewareService<S> {
    service: S,
    required_permission: String,
}

impl<S, B> Service<ServiceRequest> for RbacMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();
        let authorization = headers.get("Authorization");
    
        if authorization.is_none() {
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Missing token"))
            });
        }
    
        if let Some(auth_value) = authorization {
            if let Ok(auth_str) = auth_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
    
                    dotenv().ok();
                    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
                    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
                    let validation = Validation::default();
    
                    match decode::<Claims>(token, &decoding_key, &validation) {
                        Ok(token_data) => {
                            let user_id = token_data.claims.sub; // کپی کردن `sub` به یک متغیر جداگانه

                            // ✅ دریافت `pool` از `app_data`
                            let pool = match req.app_data::<web::Data<DbPool>>() {
                                Some(pool) => pool.clone(),
                                None => {
                                    return Box::pin(async move {
                                        Err(actix_web::error::ErrorInternalServerError("Database pool not found"))
                                    });
                                }
                            };
    
                            // اگر مقدار مجوز مورد نیاز `LOGIN` باشد، فقط لاگین بودن بررسی شود
                            if self.required_permission == "LOGIN" {
                                req.extensions_mut().insert(token_data.claims.clone()); // استفاده از clone برای جلوگیری از move
                                let fut = self.service.call(req);
                                return Box::pin(async move { fut.await });
                            }
    
                            // در غیر این صورت، مجوز را در دیتابیس بررسی کنیم
                            let permission_check = check_user_permission(&pool, user_id, &self.required_permission);
    
                            if permission_check {
                                req.extensions_mut().insert(token_data.claims.clone()); // دوباره کپی `claims` را در req ذخیره می‌کنیم
                                let fut = self.service.call(req);
                                return Box::pin(async move { fut.await });
                            } else {
                                return Box::pin(async move { Err(actix_web::error::ErrorForbidden("Forbidden")) });
                            }
                        }
                        Err(_) => {
                            return Box::pin(async move { Err(actix_web::error::ErrorUnauthorized("Invalid token")) });
                        }
                    }
                }
            }
        }
    
        Box::pin(async move {
            Err(actix_web::error::ErrorUnauthorized("Invalid token format"))
        })
    }
    
    
}

// تابع بررسی مجوز کاربر
fn check_user_permission(pool: &DbPool, user_id: i32, required_permission: &str) -> bool {
    use crate::schema::{users_roles, role_permissions, permissions};

    let mut conn = pool.get().expect("Cannot get DB connection");


    let query = diesel::dsl::select(diesel::dsl::exists(
        users_roles::table
            .inner_join(role_permissions::table.on(users_roles::role_id.eq(role_permissions::role_id)))
            .inner_join(permissions::table.on(role_permissions::permission_id.eq(permissions::id)))
            .filter(users_roles::user_id.eq(user_id))
            .filter(permissions::name.eq(required_permission))
    ));
    
    // چاپ کوئری SQL
    // let sql = debug_query::<Pg, _>(&query).to_string();
    // println!("Generated SQL: {}", sql);
    
    let exists = query
        .get_result::<bool>(&mut conn)
        .expect("Error checking permission");

    exists
}
