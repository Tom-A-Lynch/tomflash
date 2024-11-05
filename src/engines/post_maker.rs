use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

use crate::{
    config::Config,
    utils::{Result, UtilError},
    db::models::Post,
    xdotcom::types::Tweet,
};

use super::ai::Client as AIClient;

pub struct PostMaker {
    ai_client: AIClient,
    config: Config,
}

impl PostMaker {
    pub fn new(config: &Config) -> Result<Self> {
        let ai_client = AIClient::new(config)?;
        
        Ok(Self {
            ai_client,
            config: config.clone(),
        })
    }

    pub async fn generate_post(
        &self,
        short_term_memory: &str,
        long_term_memories: &[String],
        recent_posts: &[Post],
        external_context: &[String],
    ) -> Result<String> {
        // Add delay to simulate thinking/processing
        sleep(Duration::from_secs(5)).await;

        let recent_post_contents: Vec<String> = recent_posts
            .iter()
            .map(|p| p.content.clone())
            .collect();

        let content = self.ai_client
            .generate_post(
                short_term_memory,
                long_term_memories,
                &recent_post_contents,
                external_context,
            )
            .await?;

        // Clean up and validate the generated content
        let cleaned_content = self.clean_content(&content);
        
        if cleaned_content.len() > 280 {
            return Err(UtilError::ConversionError("Generated content too long".to_string()));
        }

        debug!("Generated post: {}", cleaned_content);
        Ok(cleaned_content)
    }

    fn clean_content(&self, content: &str) -> String {
        content
            .replace("Tweet:", "")
            .replace("tweet:", "")
            .trim()
            .to_string()
    }
}