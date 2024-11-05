use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use tracing::{debug, info, warn};

use crate::{
    config::Config,
    db::{Pool, models::LongTermMemory as DbMemory},
    utils::{Result, UtilError},
    utils::traits::{Embeddable, LLMFormattable},
};

use super::{
    ai::Client as AIClient,
    significance_scorer::SignificanceScorer,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub content: String,
    pub embedding: Vec<f32>,
    pub significance: f32,
    pub created_at: DateTime<Utc>,
}

pub struct LongTermMemoryEngine {
    ai_client: AIClient,
    significance_scorer: SignificanceScorer,
    config: Config,
    db_pool: Pool,
}

impl LongTermMemoryEngine {
    pub fn new(config: &Config, db_pool: Pool) -> Result<Self> {
        let ai_client = AIClient::new(config)?;
        let significance_scorer = SignificanceScorer::new(config)?;
        
        Ok(Self {
            ai_client,
            significance_scorer,
            config: config.clone(),
            db_pool,
        })
    }

    pub async fn store_memory(&self, content: &str) -> Result<DbMemory> {
        // Generate embedding
        let embedding = self.ai_client.generate_embedding(content).await?;
        
        // Calculate significance
        let significance = self.significance_scorer.score_memory(content).await?;
        
        // Only store if significance meets threshold
        if significance < self.config.memory_significance_threshold {
            debug!("Memory significance too low ({:.2}), skipping storage", significance);
            return Err(UtilError::ConversionError("Memory significance too low".to_string()));
        }

        let mut conn = self.db_pool.get().await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let memory = diesel::insert_into(crate::db::schema::long_term_memories::table)
            .values((
                crate::db::schema::long_term_memories::content.eq(content),
                crate::db::schema::long_term_memories::embedding.eq(&embedding),
                crate::db::schema::long_term_memories::significance_score.eq(significance),
            ))
            .get_result(&mut conn)
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        info!("Stored new memory with significance {:.2}", significance);
        Ok(memory)
    }

    pub async fn retrieve_relevant_memories(&self, query: &str, limit: usize) -> Result<Vec<DbMemory>> {
        let query_embedding = self.ai_client.generate_embedding(query).await?;
        
        let mut conn = self.db_pool.get().await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        // Custom SQL for vector similarity search
        let memories = diesel::sql_query(r#"
            SELECT *, 
                   (embedding <=> $1) as similarity
            FROM long_term_memories
            ORDER BY similarity DESC
            LIMIT $2
        "#)
        .bind::<diesel::sql_types::Array<diesel::sql_types::Float4>, _>(&query_embedding)
        .bind::<diesel::sql_types::Integer, _>(limit as i32)
        .load::<DbMemory>(&mut conn)
        .await
        .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        Ok(memories)
    }

    pub async fn consolidate_memories(&self) -> Result<()> {
        // Periodically consolidate similar memories
        let mut conn = self.db_pool.get().await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        // Find similar memory clusters
        let memories = diesel::sql_query(r#"
            SELECT m1.id, m1.content, m1.embedding, m2.id as similar_id
            FROM long_term_memories m1
            JOIN long_term_memories m2 ON (m1.embedding <=> m2.embedding) < 0.1
            WHERE m1.id < m2.id
        "#)
        .load::<DbMemory>(&mut conn)
        .await
        .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        // Consolidate similar memories
        for memory_pair in memories.chunks(2) {
            if memory_pair.len() != 2 {
                continue;
            }

            let consolidated_content = format!(
                "Consolidated memory: {} | {}",
                memory_pair[0].content,
                memory_pair[1].content
            );

            self.store_memory(&consolidated_content).await?;

            // Remove original memories
            diesel::delete(crate::db::schema::long_term_memories::table)
                .filter(crate::db::schema::long_term_memories::id.eq_any(&[memory_pair[0].id, memory_pair[1].id]))
                .execute(&mut conn)
                .await
                .map_err(|e| UtilError::ConversionError(e.to_string()))?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Embeddable for DbMemory {
    async fn to_embedding(&self) -> Result<Vec<f32>> {
        Ok(self.embedding.clone())
    }
}

impl LLMFormattable for DbMemory {
    fn format_for_llm(&self) -> String {
        format!(
            "[Memory: {:.2} significance] {}",
            self.significance_score,
            self.content
        )
    }
}