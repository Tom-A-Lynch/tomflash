use chrono::{DateTime, Utc};
use serde_json::Value;
use tracing::{debug, info};

use crate::{
    config::Config,
    db::{models::Post, Pool},
    utils::{Result, UtilError},
    xdotcom::{Client as TwitterClient, types::Tweet},
};

pub struct PostRetriever {
    twitter_client: TwitterClient,
    config: Config,
}

impl PostRetriever {
    pub fn new(config: &Config) -> Result<Self> {
        let twitter_client = TwitterClient::new(config)?;
        
        Ok(Self {
            twitter_client,
            config: config.clone(),
        })
    }

    pub async fn retrieve_recent_posts(&self, db: &Pool, limit: usize) -> Result<Vec<Post>> {
        let mut conn = db.get().await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        use diesel::prelude::*;
        use crate::db::schema::posts::dsl::*;

        let recent_posts = posts
            .order(created_at.desc())
            .limit(limit as i64)
            .load::<Post>(&mut conn)
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        Ok(recent_posts)
    }

    pub async fn fetch_external_context(&self, limit: usize) -> Result<Vec<String>> {
        let timeline = self.twitter_client
            .get_home_timeline(limit)
            .await?;

        let context: Vec<String> = timeline
            .iter()
            .map(|tweet| format!("@{}: {}", tweet.author_id.as_ref().unwrap_or(&"unknown".to_string()), tweet.text))
            .collect();

        debug!("Fetched {} external context items", context.len());
        Ok(context)
    }

    pub async fn fetch_notification_context(&self) -> Result<Vec<String>> {
        // This would integrate with Twitter's notification API
        // For now, returning empty vec as this requires special API access
        Ok(Vec::new())
    }

    pub fn format_post_list(posts: &[Post]) -> String {
        posts
            .iter()
            .map(|post| format!("{}: {}", post.username, post.content))
            .collect::<Vec<_>>()
            .join("\n")
    }
}