// @generated automatically by Diesel CLI.

diesel::table! {
    items (id) {
        id -> Int4,
        name -> Varchar,
        created_at -> Nullable<Timestamp>,
    }
}
