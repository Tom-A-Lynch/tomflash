use async_openai::{
    types::{CreateEmbeddingRequestArgs, CreateChatCompletionRequestArgs},
    Client as OpenAIClient,
};
use serde_json::Value;
use tracing::{debug, error, info};
use std::time::Duration;

use crate::{
    config::Config,
    utils::{Result, UtilError},
    utils::traits::{Embeddable, LLMFormattable},
};

use super::prompts::{self, PromptContext};

pub struct Client {
    openai: OpenAIClient,
    hyperbolic: reqwest::Client,
    config: Config,
}

impl Client {
    pub fn new(config: &Config) -> Result<Self> {
        let openai = OpenAIClient::new()
            .with_api_key(&config.openai_api_key);

        let hyperbolic = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        Ok(Self {
            openai,
            hyperbolic,
            config: config.clone(),
        })
    }

    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model("text-embedding-3-small")
            .input(text)
            .build()
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let response = self.openai
            .embeddings()
            .create(request)
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        Ok(response.data[0].embedding.clone())
    }

    pub async fn generate_post(
        &self,
        short_term_memory: &str,
        long_term_memories: &[String],
        recent_posts: &[String],
        external_context: &[String],
    ) -> Result<String> {
        let context = PromptContext {
            posts_data: recent_posts,
            context_data: external_context,
            memory_data: Some(short_term_memory),
        };

        let prompt = prompts::get_post_generation_prompt(&context);

        let response = self.hyperbolic
            .post("https://api.hyperbolic.xyz/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.hyperbolic_api_key))
            .json(&serde_json::json!({
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a tweet formatter. Your only job is to take the input text and format it as a tweet."
                    },
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            }))
            .send()
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        if !response.status().is_success() {
            error!("API error: {}", response.status());
            return Err(UtilError::ConversionError("API request failed".to_string()));
        }

        let response_data: Value = response.json()
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let content = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| UtilError::ConversionError("Invalid response format".to_string()))?
            .trim()
            .to_string();

        debug!("Generated post: {}", content);
        Ok(content)
    }

    pub async fn calculate_significance(&self, memory: &str) -> Result<f32> {
        let prompt = prompts::get_significance_score_prompt(memory);

        let response = self.hyperbolic
            .post("https://api.hyperbolic.xyz/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.hyperbolic_api_key))
            .json(&serde_json::json!({
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            }))
            .send()
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let response_data: Value = response.json()
            .await
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        let score_str = response_data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| UtilError::ConversionError("Invalid response format".to_string()))?;

        let score: f32 = score_str.trim()
            .parse()
            .map_err(|e| UtilError::ConversionError(e.to_string()))?;

        Ok(score / 10.0) // Normalize to 0-1 range
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        AIClient {}
        async trait AIClient {
            async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
            async fn generate_post(&self, context: &str) -> Result<String>;
            async fn calculate_significance(&self, memory: &str) -> Result<f32>;
        }
    }

    #[tokio::test]
    async fn test_significance_calculation() {
        let config = Config::from_env().unwrap();
        let client = Client::new(&config).unwrap();
        
        let score = client.calculate_significance("A truly remarkable event that changed everything")
            .await
            .unwrap();
            
        assert!(score >= 0.0 && score <= 1.0);
    }
}