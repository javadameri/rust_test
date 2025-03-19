use std::future::{Ready, ready};
use std::task::{Context, Poll};
use actix_service::{Service, Transform};
use actix_web::{Error, HttpMessage};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, Validation, DecodingKey};
use dotenv::dotenv;
use std::env;


use crate::models::user::Claims;

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtMiddlewareService { service }))
    }
}

pub struct JwtMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(& self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();
        let authorization = headers.get("Authorization");
    
        // اگر توکن وجود ندارد، درخواست رد شود


        if authorization.is_none() {
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized("Missing token"))
            });
        }
    
        if let Some(auth_value) = authorization {
            if let Ok(auth_str) = auth_value.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
    
                    dotenv().ok(); // بارگذاری متغیرهای `.env`
                    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
                    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
                    let validation = Validation::default();
    
                    match decode::<Claims>(token, &decoding_key, &validation) {
                        Ok(token_data) => {
                            req.extensions_mut().insert(token_data.claims);
                            let fut = self.service.call(req);
                            return Box::pin(async move { fut.await });
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
