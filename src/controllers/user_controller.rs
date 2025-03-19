use actix_web::{web, HttpResponse, Responder};
use diesel::prelude::*;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, Header, EncodingKey};
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};
use crate::config::DbPool;
use crate::models::user::{NewPermission, NewRole, NewUser, RolePermission, User, UserRole};
use crate::schema::{users, roles, permissions, role_permissions, users_roles};
use dotenv::dotenv;
use std::env;

#[derive(Deserialize)]
pub struct RegisterForm {
    username: String,
    password: String,
    confirm_password: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

// تابع ثبت‌نام
pub async fn register(form: web::Json<RegisterForm>, conn: web::Data<DbPool>) -> impl Responder {
    let mut conn = conn.get().expect("Error getting DB connection");

    // 1️⃣ بررسی صحت پسورد
    if form.password != form.confirm_password {
        return HttpResponse::BadRequest().body("Passwords do not match");
    }

    // 2️⃣ هش کردن پسورد
    let hashed_password = match hash(&form.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Error hashing password"),
    };

    // 3️⃣ آماده‌سازی داده برای ثبت در دیتابیس
    let new_user = NewUser {
        username: form.username.clone(), // تبدیل &String به String
        password: hashed_password.clone(),
    };

    // 4️⃣ ذخیره در پایگاه داده
    match diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&mut conn)
    {
        Ok(user) => {
            // 5️⃣ ایجاد توکن JWT
            let claims = Claims {
                sub: user.username.clone(),
                exp: chrono::Utc::now().timestamp() as usize + 60 * 60, // یک ساعت اعتبار
            };
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret("your_secret_key".as_ref()),
            )
            .unwrap();

            HttpResponse::Created().json(TokenResponse { token })
        }
        Err(_) => HttpResponse::InternalServerError().body("Error saving new user"),
    }
}

// تابع لاگین
pub async fn login(form: web::Json<LoginForm>, conn: web::Data<DbPool>) -> impl Responder {
    use crate::schema::users::dsl::*;
    let mut conn = conn.get().expect("Error getting DB connection"); // دریافت اتصال متغیر

    // جستجوی کاربر بر اساس نام کاربری
    let user_result = users.filter(username.eq(&form.username))
        .first::<User>(&mut conn);

    match user_result {
        Ok(user) => {
            // تایید رمز عبور وارد شده با رمز عبور ذخیره شده
            if verify(&form.password, &user.password).unwrap_or(false) {
                let claims = Claims {
                    sub: user.username,
                    exp: (Utc::now() + Duration::days(1)).timestamp() as usize,
                };
                dotenv().ok(); // بارگذاری متغیرهای `.env`
                let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

                let encoding_key = EncodingKey::from_secret(secret_key.as_ref());
                let token = match encode(&Header::default(), &claims, &encoding_key) {
                    Ok(t) => t,
                    Err(_) => return HttpResponse::InternalServerError().body("Error generating token"),
                };

                HttpResponse::Created().json(TokenResponse { token })
            } else {
                HttpResponse::Unauthorized().body("Invalid credentials")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Invalid credentials"),
    }
}

// تابع افزودن نقش
pub async fn add_role(form: web::Json<NewRole>, conn: web::Data<DbPool>) -> impl Responder {
    let mut conn = conn.get().expect("Error getting DB connection"); // دریافت اتصال متغیر

    let new_role = NewRole {
        name: form.name.clone(),
        role_type: form.role_type.clone(), // نوع پیش‌فرض
    };

    diesel::insert_into(roles::table)
        .values(&new_role)
        .execute(&mut conn)
        .expect("Error saving new role");

    HttpResponse::Created().body("Role added successfully")
}

// تابع افزودن دسترسی
pub async fn add_permission(form: web::Json<NewPermission>, conn: web::Data<DbPool>) -> impl Responder {
    let mut conn = conn.get().expect("Error getting DB connection"); // دریافت اتصال متغیر


    let new_permission = NewPermission {
        name: form.name.clone(),
        permission_type: form.permission_type.clone()
    };

    diesel::insert_into(permissions::table)
        .values(&new_permission)
        .execute(&mut conn)
        .expect("Error saving new permission");

    HttpResponse::Created().body("Permission added successfully")
}

// تابع افزودن دسترسی به نقش
pub async fn add_role_permission(form: web::Json<(i32, i32)>, conn: web::Data<DbPool>) -> impl Responder {
    let mut conn = conn.get().expect("Error getting DB connection"); // دریافت اتصال متغیر

    let (role_id, permission_id) = form.into_inner();

    diesel::insert_into(role_permissions::table)
        .values((role_permissions::role_id.eq(role_id), role_permissions::permission_id.eq(permission_id)))
        .execute(&mut conn)
        .expect("Error adding role permission");

    HttpResponse::Created().body("Role Permission added successfully")
}

// تابع اختصاص نقش به کاربر
pub async fn assign_role_to_user(form: web::Json<(i32, i32)>, conn: web::Data<DbPool>) -> impl Responder {
    let mut conn = conn.get().expect("Error getting DB connection"); // دریافت اتصال متغیر

    let (user_id, role_id) = form.into_inner();

    diesel::insert_into(users_roles::table)
        .values((users_roles::user_id.eq(user_id), users_roles::role_id.eq(role_id)))
        .execute(&mut conn)
        .expect("Error assigning role to user");

    HttpResponse::Created().body("Role assigned to user successfully")
}

// تابع دریافت دسترسی‌های یک نقش
pub async fn get_permissions_for_role(role_path: web::Path<i32>, conn: web::Data<DbPool>) -> impl Responder {
    let mut conn = conn.get().expect("Error getting DB connection"); // دریافت اتصال از Pool

    use crate::schema::role_permissions::dsl::*;

    let role_param = role_path.into_inner(); // مقدار عددی role_id را دریافت کنید

    let permissions = role_permissions
        .filter(role_id.eq(role_param)) // اینجا دیگر مشکل نخواهید داشت
        .load::<RolePermission>(&mut conn)
        .expect("Error loading permissions");

    HttpResponse::Ok().json(permissions)
}



pub async fn get_roles_for_user(path_user_id: web::Path<i32>, conn: web::Data<DbPool>) -> impl Responder {
    let mut conn = conn.get().expect("Error getting DB connection"); // دریافت اتصال متغیر

    use crate::schema::users_roles::dsl::*;

    let user_param = path_user_id.into_inner(); // مقدار `user_id` از مسیر را دریافت می‌کنیم

    let roles = users_roles
        .filter(user_id.eq(user_param)) // حالا مقدار مسیر را مقایسه می‌کنیم
        .load::<UserRole>(&mut conn)
        .expect("Error loading roles");

    HttpResponse::Ok().json(roles)
}


