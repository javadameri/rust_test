-- This file should undo anything in `up.sql`
-- حذف روابط کاربران و نقش‌ها
DROP TABLE IF EXISTS users_roles;

-- حذف روابط نقش‌ها و دسترسی‌ها
DROP TABLE IF EXISTS role_permissions;

-- حذف جداول دسترسی‌ها و نقش‌ها
DROP TABLE IF EXISTS permissions;
DROP TABLE IF EXISTS roles;

-- حذف جدول کاربران
DROP TABLE IF EXISTS users;
