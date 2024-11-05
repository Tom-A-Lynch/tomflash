use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    pub author_id: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub conversation_id: Option<String>,
    pub in_reply_to_user_id: Option<String>,
    pub referenced_tweets: Option<Vec<ReferencedTweet>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReferencedTweet {
    pub id: String,
    pub type_: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TweetResponse {
    pub data: Tweet,
    pub includes: Option<Includes>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimelineResponse {
    pub data: Vec<Tweet>,
    pub includes: Option<Includes>,
    pub meta: Option<Meta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Includes {
    pub users: Option<Vec<User>>,
    pub tweets: Option<Vec<Tweet>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Meta {
    pub result_count: i32,
    pub newest_id: Option<String>,
    pub oldest_id: Option<String>,
    pub next_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserResponse {
    pub data: User,
}

#[derive(Debug, Clone, Default)]
pub struct AuthTokens {
    pub user_id: String,
    pub ct0: String,
    pub auth_token: String,
}

impl AuthTokens {
    pub fn new(user_id: String, ct0: String, auth_token: String) -> Self {
        Self {
            user_id,
            ct0,
            auth_token,
        }
    }
}