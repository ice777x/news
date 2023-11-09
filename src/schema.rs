// @generated automatically by Diesel CLI.

diesel::table! {
    feeds (id) {
        id -> Int4,
        title -> Text,
        link -> Nullable<Text>,
        description -> Nullable<Text>,
        author -> Nullable<Text>,
        image -> Nullable<Text>,
        content -> Nullable<Text>,
        published -> Nullable<Text>,
    }
}
