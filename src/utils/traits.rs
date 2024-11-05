use async_trait::async_trait;
use serde::Serialize;

/// Trait for types that can be converted to embeddings
#[async_trait]
pub trait Embeddable {
    async fn to_embedding(&self) -> super::Result<Vec<f32>>;
}

/// Trait for types that can be formatted for LLM input
pub trait LLMFormattable {
    fn format_for_llm(&self) -> String;
}

/// Trait for types that can be scored for significance
#[async_trait]
pub trait Scorable {
    async fn calculate_significance(&self) -> super::Result<f32>;
}

/// Helper trait for types that need JSON serialization with specific requirements
pub trait JsonFormatter: Serialize {
    fn to_json_string(&self) -> super::Result<String> {
        serde_json::to_string(self)
            .map_err(super::UtilError::from)
    }

    fn to_json_pretty(&self) -> super::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(super::UtilError::from)
    }
}

// Blanket implementation for any type that implements Serialize
impl<T: Serialize> JsonFormatter for T {}