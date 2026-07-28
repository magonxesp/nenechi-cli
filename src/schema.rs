diesel::table! {
    wallpapers (id) {
        id -> Varchar,
        pixiv_illustration_id -> Nullable<Varchar>,
        tags -> Text,
        aspect_ratio -> Varchar,
        path -> Text,
        file_name -> Varchar
    }
}

diesel::table! {
    series_metadata (id) {
        id -> Varchar,
        title -> Varchar,
        path -> Text,
        season -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(series_metadata, wallpapers,);
