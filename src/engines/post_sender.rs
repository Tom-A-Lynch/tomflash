use tracing::{debug, error, info};

use crate::{
    config::Config,
    db::{models::Post, Pool},
    utils::{Result, UtilError},
    xdotcom::Client as TwitterClient,
};

pub struct PostSender {
    twitter_client: TwitterClient,
    config: Config,
}

impl PostSender {
    pub fn new(config: &Config) -> Result<Self> {
        let twitter_client = TwitterClient::new(config)?;
        
        Ok(Self {
            twitter_client,
            config: config.clone(),
        })
    }

    pub async fn send_post(&self, content: &str) -> Result<String> {
        info!("Sending post: {}", content);
        
        let tweet = self.twitter_client
            .post_tweet(content)
            .await?;

        debug!("Post sent successfully, id: {}", tweet.id);
        Ok(tweet.id)
    }

    pub async fn reply_to_post(&self, content: &str, reply_to_id: &str) -> Result<String> {
        info!("Sending reply to {}: {}", reply_to_id, content);
        
        let tweet = self.twitter_client
            .reply_to_tweet(content, reply_to_id)
            .await?;

        debug!("Reply sent successfully, id: {}", tweet.id);
        Ok(tweet.id)
    }

    pub async fn store_post(&self, db: &Pool, post: Post) -> Result<Post> {
        let mut conn = db.get().await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        use diesel::prelude::*;
        use crate::db::schema::posts::dsl::*;

        let stored_post = diesel::insert_into(posts)
            .values(&post)
            .get_result(&mut conn)
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        Ok(stored_post)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        TwitterClient {}
        async trait TwitterClient {
            async fn post_tweet(&self, content: &str) -> Result<String>;
            async fn reply_to_tweet(&self, content: &str, reply_to_id: &str) -> Result<String>;
        }
    }

    #[tokio::test]
    async fn test_post_sending() {
        let mut mock_client = MockTwitterClient::new();
        mock_client
            .expect_post_tweet()
            .with(eq("test post"))
            .returning(|_| Ok("tweet_id_123".to_string()));

        // Test implementation here
    }
}