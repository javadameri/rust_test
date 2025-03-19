use diesel::prelude::*;
use crate::schema::{roles, permissions, role_permissions, users_roles, users};
use serde::{Serialize, Deserialize};


#[derive(Queryable, Insertable, Identifiable, Debug)] // حذف Associations
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Clone)] // اضافه کردن Clone برای امکان کپی کردن
pub struct Claims {
    pub sub: i32,  // شناسه کاربر (id) به جای نام کاربری
    pub exp: usize,   // زمان انقضای توکن
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Queryable, Insertable, Identifiable)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub role_type: String
}

#[derive(Insertable, Serialize, Deserialize)]
#[diesel(table_name = roles)]
pub struct NewRole {
    pub name: String,
    pub role_type: String
}

#[derive(Queryable, Insertable, Identifiable)]
#[diesel(table_name = permissions)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub permission_type: String
}

#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = permissions)]
pub struct NewPermission {
    pub name: String,
    pub permission_type: String
}

#[derive(Queryable, Insertable, Associations, Serialize, Deserialize)]
#[diesel(table_name = role_permissions)]
#[diesel(belongs_to(Permission))]
#[diesel(belongs_to(Role))]
pub struct RolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Queryable, Insertable, Associations, Serialize, Deserialize)]
#[diesel(table_name = users_roles)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Role))]
pub struct UserRole {
    pub user_id: i32,
    pub role_id: i32
}
