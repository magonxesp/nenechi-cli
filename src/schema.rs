diesel::table! {
    wallpapers (id) {
        id -> Varchar,
        pixiv_illustration_id -> Nullable<Varchar>,
        tags -> Text,
        aspect_ratio -> Varchar,
    }
}
