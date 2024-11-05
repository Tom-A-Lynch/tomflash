use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{debug, info};

use crate::{
    config::Config,
    utils::{Result, UtilError},
    utils::traits::{Embeddable, LLMFormattable},
    db::models::Post,
};

use super::{
    ai::Client as AIClient,
    prompts::PromptContext,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTermMemory {
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub context_vector: Vec<f32>,
    pub source_type: MemorySourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemorySourceType {
    ExternalContext,
    InternalThought,
    Interaction,
    Observation,
}

pub struct ShortTermMemoryEngine {
    ai_client: AIClient,
    config: Config,
    recent_memories: VecDeque<ShortTermMemory>,
    max_memories: usize,
}

impl ShortTermMemoryEngine {
    pub fn new(config: &Config) -> Result<Self> {
        let ai_client = AIClient::new(config)?;
        
        Ok(Self {
            ai_client,
            config: config.clone(),
            recent_memories: VecDeque::with_capacity(100),
            max_memories: 100, // Configurable memory limit
        })
    }

    pub async fn process_current_context(
        &mut self,
        posts: &[Post],
        external_context: &[String],
    ) -> Result<String> {
        let context = PromptContext {
            posts_data: &posts.iter()
                .map(|p| p.content.clone())
                .collect::<Vec<_>>(),
            context_data: external_context,
            memory_data: None,
        };

        // Generate internal monologue about current context
        let thought = self.ai_client
            .generate_post(
                "",
                &[],
                &context.posts_data,
                &context.context_data,
            )
            .await?;

        // Create embedding for the thought
        let embedding = self.ai_client.generate_embedding(&thought).await?;

        // Store in short-term memory
        let memory = ShortTermMemory {
            content: thought.clone(),
            timestamp: Utc::now(),
            context_vector: embedding,
            source_type: MemorySourceType::InternalThought,
        };

        self.add_memory(memory);

        Ok(thought)
    }

    pub async fn find_relevant_context(&self, query: &str) -> Result<Vec<ShortTermMemory>> {
        let query_embedding = self.ai_client.generate_embedding(query).await?;
        
        let mut memories_with_scores: Vec<(f32, &ShortTermMemory)> = self.recent_memories
            .iter()
            .map(|memory| {
                let similarity = cosine_similarity(&query_embedding, &memory.context_vector);
                (similarity, memory)
            })
            .collect();

        // Sort by similarity score
        memories_with_scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Return top 5 most relevant memories
        Ok(memories_with_scores
            .into_iter()
            .take(5)
            .map(|(_, memory)| memory.clone())
            .collect())
    }

    fn add_memory(&mut self, memory: ShortTermMemory) {
        if self.recent_memories.len() >= self.max_memories {
            self.recent_memories.pop_front();
        }
        self.recent_memories.push_back(memory);
    }
}

#[async_trait::async_trait]
impl Embeddable for ShortTermMemory {
    async fn to_embedding(&self) -> Result<Vec<f32>> {
        Ok(self.context_vector.clone())
    }
}

impl LLMFormattable for ShortTermMemory {
    fn format_for_llm(&self) -> String {
        format!("[{}] {}", self.timestamp.format("%Y-%m-%d %H:%M:%S"), self.content)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_relevance() {
        let config = Config::from_env().unwrap();
        let mut engine = ShortTermMemoryEngine::new(&config).unwrap();

        // Add test memories
        let memory = ShortTermMemory {
            content: "Test memory about AI".to_string(),
            timestamp: Utc::now(),
            context_vector: vec![0.1; 1536], // Example embedding size
            source_type: MemorySourceType::InternalThought,
        };
        engine.add_memory(memory);

        let relevant = engine.find_relevant_context("AI thoughts").await.unwrap();
        assert!(!relevant.is_empty());
    }
}