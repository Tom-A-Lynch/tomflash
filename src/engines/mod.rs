use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

pub mod ai;
pub mod post_maker;
pub mod post_retriever;
pub mod post_sender;
pub mod prompts;
pub mod short_term_mem;
pub mod long_term_mem;
pub mod significance_scorer;
pub mod wallet;

use crate::{
    config::Config,
    db::Pool,
    utils::{Result, UtilError},
    xdotcom::types::Tweet,
};

pub struct EngineManager {
    short_term: Arc<Mutex<short_term_mem::ShortTermMemoryEngine>>,
    long_term: Arc<long_term_mem::LongTermMemoryEngine>,
    post_maker: Arc<post_maker::PostMaker>,
    post_retriever: Arc<post_retriever::PostRetriever>,
    post_sender: Arc<post_sender::PostSender>,
    significance: Arc<significance_scorer::SignificanceScorer>,
    wallet: Arc<wallet::Client>,
    ai: Arc<ai::Client>,
    config: Config,
}

impl EngineManager {
    pub async fn new(config: &Config, db_pool: Pool) -> Result<Self> {
        let ai_client = Arc::new(ai::Client::new(config)?);
        
        let short_term = Arc::new(Mutex::new(
            short_term_mem::ShortTermMemoryEngine::new(config)?
        ));
        
        let long_term = Arc::new(
            long_term_mem::LongTermMemoryEngine::new(config, db_pool.clone())?
        );
        
        Ok(Self {
            short_term,
            long_term,
            post_maker: Arc::new(post_maker::PostMaker::new(config)?),
            post_retriever: Arc::new(post_retriever::PostRetriever::new(config)?),
            post_sender: Arc::new(post_sender::PostSender::new(config)?),
            significance: Arc::new(significance_scorer::SignificanceScorer::new(config)?),
            wallet: Arc::new(wallet::Client::new(config)?),
            ai: ai_client,
            config: config.clone(),
        })
    }

    pub async fn process_cognitive_cycle(&self, db: &Pool) -> Result<Option<String>> {
        // 1. Gather context
        let recent_posts = self.post_retriever
            .retrieve_recent_posts(db, 10)
            .await?;
            
        let external_context = self.post_retriever
            .fetch_external_context(20)
            .await?;

        // 2. Process in short-term memory
        let current_thought = {
            let mut short_term = self.short_term.lock().await;
            short_term.process_current_context(&recent_posts, &external_context).await?
        };

        // 3. Check memory significance
        let significance = self.significance
            .score_memory(&current_thought)
            .await?;

        // 4. Store significant memories
        if significance > self.config.memory_significance_threshold {
            self.long_term
                .store_memory(&current_thought)
                .await?;
            info!("Stored significant memory: {:.2} significance", significance);
        }

        // 5. Retrieve relevant long-term memories
        let relevant_memories = self.long_term
            .retrieve_relevant_memories(&current_thought, 5)
            .await?;

        // 6. Generate post if conditions are met
        if should_generate_post(&current_thought, significance) {
            let memory_contexts: Vec<String> = relevant_memories
                .iter()
                .map(|m| m.content.clone())
                .collect();

            let post_content = self.post_maker
                .generate_post(
                    &current_thought,
                    &memory_contexts,
                    &recent_posts,
                    &external_context,
                )
                .await?;

            return Ok(Some(post_content));
        }

        Ok(None)
    }

    pub async fn handle_interaction(&self, tweet: &Tweet) -> Result<Option<String>> {
        // Process mentions and replies
        let notification_context = self.post_retriever
            .fetch_notification_context()
            .await?;

        // Check if interaction requires response
        if should_respond_to_tweet(tweet) {
            let relevant_memories = self.long_term
                .retrieve_relevant_memories(&tweet.text, 3)
                .await?;

            let memory_context: Vec<String> = relevant_memories
                .iter()
                .map(|m| m.content.clone())
                .collect();

            // Generate response
            let response = self.post_maker
                .generate_post(
                    &tweet.text,
                    &memory_context,
                    &[], // No recent posts needed for direct replies
                    &[tweet.text.clone()],
                )
                .await?;

            return Ok(Some(response));
        }

        Ok(None)
    }

    pub async fn consolidate_memories(&self) -> Result<()> {
        info!("Starting memory consolidation...");
        self.long_term.consolidate_memories().await?;
        Ok(())
    }

    pub async fn check_wallet_interactions(&self, content: &str) -> Result<Option<String>> {
        if let Some(address) = wallet::wallet_address_in_post(content).await {
            let balance = self.wallet.get_balance().await?;
            
            if balance > self.config.min_transaction_balance {
                // Process potential transaction
                // Implementation depends on specific rules
                debug!("Found wallet interaction: {}", address);
            }
        }
        Ok(None)
    }
}

fn should_generate_post(thought: &str, significance: f32) -> bool {
    // Basic heuristic - can be made more sophisticated
    significance > 0.6 && thought.len() > 20
}

fn should_respond_to_tweet(tweet: &Tweet) -> bool {
    // Basic heuristic - can be made more sophisticated
    tweet.text.contains("@yourbotname") || 
    tweet.in_reply_to_user_id.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cognitive_cycle() {
        let config = Config::from_env().unwrap();
        let db_pool = crate::db::establish_connection(&config).await.unwrap();
        
        let engine = EngineManager::new(&config, db_pool.clone()).await.unwrap();
        let result = engine.process_cognitive_cycle(&db_pool).await;
        
        assert!(result.is_ok());
    }
}