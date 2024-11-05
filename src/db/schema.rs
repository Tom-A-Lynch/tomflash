// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        content -> Text,
        user_id -> Int4,
        username -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        post_type -> Varchar,
        comment_count -> Int4,
        image_path -> Nullable<Varchar>,
        tweet_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    long_term_memories (id) {
        id -> Int4,
        content -> Text,
        embedding -> Array<Float4>,
        significance_score -> Float4,
        created_at -> Timestamptz,
    }
}

diesel::joinable!(posts -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    posts,
    long_term_memories,
);