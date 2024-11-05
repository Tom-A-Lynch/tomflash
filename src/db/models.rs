use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::db::schema::*;

#[derive(Debug, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
}

#[derive(Debug, Queryable, Selectable, Identifiable, Associations, Serialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = posts)]
pub struct Post {
    pub id: i32,
    pub content: String,
    pub user_id: i32,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub post_type: String,
    pub comment_count: i32,
    pub image_path: Option<String>,
    pub tweet_id: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub content: String,
    pub user_id: i32,
    pub username: String,
    pub post_type: String,
    pub image_path: Option<String>,
    pub tweet_id: Option<String>,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = long_term_memories)]
pub struct LongTermMemory {
    pub id: i32,
    pub content: String,
    pub embedding: Vec<f32>,
    pub significance_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = long_term_memories)]
pub struct NewLongTermMemory {
    pub content: String,
    pub embedding: Vec<f32>,
    pub significance_score: f32,
}

impl User {
    pub async fn create(
        pool: &DbPool,
        new_user: NewUser,
    ) -> QueryResult<User> {
        use crate::db::schema::users::dsl::*;
        
        let mut conn = pool.get().await?;
        diesel::insert_into(users)
            .values(&new_user)
            .get_result(&mut conn)
            .await
    }
    
    pub async fn find_by_username(
        pool: &DbPool,
        username_query: &str,
    ) -> QueryResult<Option<User>> {
        use crate::db::schema::users::dsl::*;
        
        let mut conn = pool.get().await?;
        users
            .filter(username.eq(username_query))
            .first(&mut conn)
            .await
            .optional()
    }
}

// Similar implementations for Post and LongTermMemory...